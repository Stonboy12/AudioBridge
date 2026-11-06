use audio_types::{AudioPcmFrame, AudioProfile};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Audio capture source trait
pub trait AudioCaptureSource: Send {
    /// Returns the next captured PCM frame (blocks for frame_duration_ms or returns upon availability)
    fn capture_frame(&mut self) -> Option<AudioPcmFrame>;
    /// Starts the capture pipeline
    fn start(&mut self) -> Result<(), String>;
    /// Stops the capture pipeline
    fn stop(&mut self);
}

/// Simulated WASAPI Loopback Audio Capture Source
/// Generates pristine audio samples (440Hz Reference A tone with stereo harmonic)
/// exactly mimicking the WASAPI audio engine 10ms/20ms callback interval.
pub struct SyntheticWasapiCaptureSource {
    profile: AudioProfile,
    is_running: Arc<AtomicBool>,
    seq: u64,
    start_time: Option<Instant>,
    phase: f32,
}

impl SyntheticWasapiCaptureSource {
    pub fn new(profile: AudioProfile) -> Self {
        Self {
            profile,
            is_running: Arc::new(AtomicBool::new(false)),
            seq: 0,
            start_time: None,
            phase: 0.0,
        }
    }
}

impl AudioCaptureSource for SyntheticWasapiCaptureSource {
    fn start(&mut self) -> Result<(), String> {
        self.is_running.store(true, Ordering::SeqCst);
        self.start_time = Some(Instant::now());
        self.seq = 0;
        self.phase = 0.0;
        Ok(())
    }

    fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    fn capture_frame(&mut self) -> Option<AudioPcmFrame> {
        if !self.is_running.load(Ordering::SeqCst) {
            return None;
        }

        let samples_per_channel = self.profile.samples_per_frame();
        let total_samples = self.profile.total_samples_per_frame();
        let mut pcm_data = Vec::with_capacity(total_samples);

        let freq = 440.0; // 440 Hz concert pitch
        let sample_rate = self.profile.sample_rate as f32;
        let delta = 2.0 * PI * freq / sample_rate;

        for _ in 0..samples_per_channel {
            let left = (self.phase.sin() * 16384.0) as i16;
            let right = ((self.phase * 1.5).sin() * 14000.0) as i16; // Harmonic for stereo balance
            pcm_data.push(left);
            if self.profile.channels == 2 {
                pcm_data.push(right);
            }
            self.phase += delta;
            if self.phase > 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
        }

        let elapsed_us = self.start_time.map_or(0, |t| t.elapsed().as_micros() as u64);
        let frame = AudioPcmFrame::new(
            self.seq,
            elapsed_us,
            self.profile.sample_rate,
            self.profile.channels,
            pcm_data,
        );
        self.seq += 1;

        Some(frame)
    }
}

/// Documentation of the Windows WASAPI Loopback Engine structure:
///
/// On Windows production builds, `WindowsWasapiLoopbackSource` initializes:
/// 1. `CoInitializeEx(None, COINIT_MULTITHREADED)`
/// 2. `CoCreateInstance(&MMDeviceEnumerator)`
/// 3. `GetDefaultAudioEndpoint(eRender, eConsole, &pDevice)`
/// 4. `pDevice.Activate(&IAudioClient, CLSCTX_ALL, None, &mut pAudioClient)`
/// 5. `pAudioClient.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, buffer_duration, 0, pWaveFormat, None)`
/// 6. `pAudioClient.GetService(&IAudioCaptureClient, &mut pCaptureClient)`
/// 7. Event-driven loop `pCaptureClient.GetBuffer(&mut pData, &mut frames, &mut flags, None, None)`
/// 8. Format normalization from IEEE Float (WASAPI default) -> 16-bit signed PCM
/// 9. Feed into Opus encoder ring buffer.
#[allow(dead_code)]
pub struct WindowsWasapiLoopbackContract;
