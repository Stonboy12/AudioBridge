use audio_capture::{AudioCaptureEngine, CaptureError, SyntheticLoopbackCapture};
use audio_codec::{create_encoder, AudioEncoder, CodecError};
use audio_processing::{float32_to_pcm16, FrameAssembler, Resampler};
use audio_types::{AudioProfile, CodecType};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Capture error: {0}")]
    Capture(#[from] CaptureError),
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("Pipeline already running")]
    AlreadyRunning,
    #[error("Pipeline not running")]
    NotRunning,
    #[error("Processing error: {0}")]
    Processing(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedAudioPacket {
    pub sequence_number: u64,
    pub timestamp_us: u64,
    pub codec: CodecType,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub is_running: bool,
    pub active_device_name: String,
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate_kbps: u32,
    pub frames_captured: u64,
    pub frames_encoded: u64,
    pub frames_dropped_overflow: u64,
    pub session_duration_secs: u64,
    pub queue_depth_frames: usize,
    pub average_encode_time_us: u32,
}

struct PipelineSharedState {
    assembler: FrameAssembler,
    resampler: Option<Resampler>,
    encoder: Box<dyn AudioEncoder>,
    packet_queue: VecDeque<EncodedAudioPacket>,
    max_queue_frames: usize,
    temp_pcm16: Vec<i16>,
    encoded_scratch: Vec<u8>,
    total_encode_time_us: u64,
    encode_count: u64,
}

pub struct AudioPipeline {
    profile: AudioProfile,
    capture_engine: Arc<dyn AudioCaptureEngine>,
    state: Arc<Mutex<PipelineSharedState>>,
    is_running: Arc<AtomicBool>,
    start_time: Mutex<Option<Instant>>,
    frames_encoded_counter: Arc<AtomicU64>,
    frames_dropped_counter: Arc<AtomicU64>,
}

impl AudioPipeline {
    pub fn new(profile: AudioProfile, capture_engine: Arc<dyn AudioCaptureEngine>) -> Result<Self, PipelineError> {
        let device = capture_engine.active_device();
        let resampler = if device.sample_rate != profile.sample_rate {
            Some(
                Resampler::new(device.sample_rate, profile.sample_rate, profile.channels)
                    .map_err(|e| PipelineError::Processing(e.to_string()))?,
            )
        } else {
            None
        };

        let assembler = FrameAssembler::new(profile.clone());
        let encoder = create_encoder(&profile)?;

        let state = Arc::new(Mutex::new(PipelineSharedState {
            assembler,
            resampler,
            encoder,
            packet_queue: VecDeque::with_capacity(100),
            max_queue_frames: 50, // 1 second buffer max
            temp_pcm16: Vec::with_capacity(4096),
            encoded_scratch: vec![0u8; 4096],
            total_encode_time_us: 0,
            encode_count: 0,
        }));

        Ok(Self {
            profile,
            capture_engine,
            state,
            is_running: Arc::new(AtomicBool::new(false)),
            start_time: Mutex::new(None),
            frames_encoded_counter: Arc::new(AtomicU64::new(0)),
            frames_dropped_counter: Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn new_with_synthetic(profile: AudioProfile) -> Result<Self, PipelineError> {
        let capture = Arc::new(SyntheticLoopbackCapture::new(profile.sample_rate, profile.channels));
        Self::new(profile, capture)
    }

    pub fn start(&self) -> Result<(), PipelineError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(PipelineError::AlreadyRunning);
        }

        *self.start_time.lock() = Some(Instant::now());

        let state = self.state.clone();
        let frames_encoded = self.frames_encoded_counter.clone();
        let frames_dropped = self.frames_dropped_counter.clone();
        let is_running = self.is_running.clone();
        let start_instant = Instant::now();

        let callback = Arc::new(move |float_samples: &[f32], _frames: usize| {
            if !is_running.load(Ordering::Relaxed) {
                return;
            }

            let mut lock = state.lock();
            let current_time_us = start_instant.elapsed().as_micros() as u64;

            // 1. Normalization Float32 -> PCM16
            if lock.temp_pcm16.len() < float_samples.len() {
                lock.temp_pcm16.resize(float_samples.len(), 0);
            }
            float32_to_pcm16(float_samples, &mut lock.temp_pcm16[..float_samples.len()]);
            let pcm16_slice = &lock.temp_pcm16[..float_samples.len()];

            // 2. Resampling if needed
            let processed_samples = if let Some(ref resampler) = lock.resampler {
                resampler.resample(pcm16_slice)
            } else {
                pcm16_slice.to_vec()
            };

            // 3. Assemble frames
            lock.assembler.push_samples(&processed_samples);

            let mut ready_frames = Vec::new();
            while let Some(frame) = lock.assembler.pop_frame(current_time_us) {
                ready_frames.push(frame);
            }

            for frame in ready_frames {
                let encode_start = Instant::now();
                let PipelineSharedState {
                    ref mut encoder,
                    ref mut encoded_scratch,
                    ref mut packet_queue,
                    max_queue_frames,
                    ref mut total_encode_time_us,
                    ref mut encode_count,
                    ..
                } = *lock;

                match encoder.encode(&frame.data, encoded_scratch) {
                    Ok(bytes) => {
                        let encode_duration_us = encode_start.elapsed().as_micros() as u64;
                        *total_encode_time_us += encode_duration_us;
                        *encode_count += 1;

                        let packet = EncodedAudioPacket {
                            sequence_number: frame.sequence_number,
                            timestamp_us: frame.timestamp_us,
                            codec: encoder.codec_type(),
                            payload: encoded_scratch[..bytes].to_vec(),
                        };

                        if packet_queue.len() >= max_queue_frames {
                            // Backpressure: drop oldest frame to preserve live low latency
                            packet_queue.pop_front();
                            frames_dropped.fetch_add(1, Ordering::Relaxed);
                        }

                        packet_queue.push_back(packet);
                        frames_encoded.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        log::error!("Pipeline encoder error: {}", e);
                    }
                }
            }
        });

        self.capture_engine.start(callback)?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), PipelineError> {
        if !self.is_running.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        self.capture_engine.stop()?;
        let mut lock = self.state.lock();
        lock.assembler.reset();
        lock.packet_queue.clear();
        Ok(())
    }

    pub fn pop_packet(&self) -> Option<EncodedAudioPacket> {
        self.state.lock().packet_queue.pop_front()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> PipelineStatus {
        let lock = self.state.lock();
        let running = self.is_running.load(Ordering::SeqCst);
        let duration = self.start_time.lock().map_or(0, |t| t.elapsed().as_secs());
        let device = self.capture_engine.active_device();

        let avg_encode_us = if lock.encode_count > 0 {
            (lock.total_encode_time_us / lock.encode_count) as u32
        } else {
            0
        };

        PipelineStatus {
            is_running: running,
            active_device_name: device.name,
            codec: format!("{:?}", self.profile.codec),
            sample_rate: self.profile.sample_rate,
            channels: self.profile.channels,
            bitrate_kbps: self.profile.bitrate_kbps,
            frames_captured: self.capture_engine.frames_captured(),
            frames_encoded: self.frames_encoded_counter.load(Ordering::Relaxed),
            frames_dropped_overflow: self.frames_dropped_counter.load(Ordering::Relaxed),
            session_duration_secs: duration,
            queue_depth_frames: lock.packet_queue.len(),
            average_encode_time_us: avg_encode_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_pipeline_streaming_lifecycle_and_packet_generation() {
        let profile = AudioProfile::default();
        let pipeline = AudioPipeline::new_with_synthetic(profile).expect("Pipeline creation failed");

        assert!(!pipeline.is_running());
        pipeline.start().expect("Failed to start pipeline");
        assert!(pipeline.is_running());

        // Run for 60 ms to produce ~3 frames (20ms each)
        thread::sleep(Duration::from_millis(60));

        let status = pipeline.status();
        assert!(status.frames_captured > 0);
        assert!(status.frames_encoded > 0);

        let packet = pipeline.pop_packet();
        assert!(packet.is_some());
        let p = packet.unwrap();
        assert_eq!(p.sequence_number, 0);
        assert!(p.payload.len() > 20);

        pipeline.stop().expect("Failed to stop pipeline");
        assert!(!pipeline.is_running());
    }
}
