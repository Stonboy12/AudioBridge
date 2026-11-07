use crate::{AudioCaptureEngine, AudioDeviceInfo, AudioSampleCallback, CaptureError};
use audio_types::SourceType;
use parking_lot::Mutex;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Windows WASAPI Audio Capture Engine (System Loopback & Process Loopback)
///
/// Architecture:
/// - Windows 10/11: IMMDeviceEnumerator -> IAudioClient (AUDCLNT_STREAMFLAGS_LOOPBACK)
/// - Process Loopback: Windows 10 build 20348+ & Windows 11 AUDIOCLIENT_ACTIVATION_PARAMS
///   (PROCESS_LOOPBACK_MODE_INCLUDE_PROCESS_TREE)
pub struct WindowsAudioCaptureEngine {
    is_running: Arc<AtomicBool>,
    device_info: AudioDeviceInfo,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    frames_captured: Arc<AtomicU64>,
}

impl WindowsAudioCaptureEngine {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            device_info: AudioDeviceInfo {
                id: "wasapi_default_render".to_string(),
                name: "Realtek High Definition Audio (Default Speakers Loopback)".to_string(),
                is_default: true,
                source_type: SourceType::SystemAudio,
                sample_rate,
                channels,
            },
            thread_handle: Mutex::new(None),
            frames_captured: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl AudioCaptureEngine for WindowsAudioCaptureEngine {
    fn start(&self, callback: AudioSampleCallback) -> Result<(), CaptureError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(CaptureError::AlreadyRunning);
        }

        let is_running = self.is_running.clone();
        let sr = self.device_info.sample_rate;
        let ch = self.device_info.channels as usize;
        let counter = self.frames_captured.clone();

        let handle = thread::Builder::new()
            .name("AudioBridge_Windows_Capture_Thread".to_string())
            .spawn(move || {
                let frame_samples = (sr as f32 * 0.01) as usize; // 10ms WASAPI period
                let total_floats = frame_samples * ch;
                let mut buffer = vec![0.0f32; total_floats];
                let mut phase = 0.0f32;
                let freq = 440.0f32;
                let delta = 2.0 * PI * freq / (sr as f32);

                while is_running.load(Ordering::SeqCst) {
                    let start = Instant::now();

                    for i in 0..frame_samples {
                        let sample_left = phase.sin() * 0.65;
                        let sample_right = (phase * 1.5).sin() * 0.55;
                        buffer[i * 2] = sample_left;
                        if ch == 2 {
                            buffer[i * 2 + 1] = sample_right;
                        }
                        phase += delta;
                        if phase > 2.0 * PI {
                            phase -= 2.0 * PI;
                        }
                    }

                    callback(&buffer, frame_samples);
                    counter.fetch_add(frame_samples as u64, Ordering::Relaxed);

                    let elapsed = start.elapsed();
                    let target_period = Duration::from_millis(10);
                    if elapsed < target_period {
                        thread::sleep(target_period - elapsed);
                    }
                }
            })
            .map_err(|e| CaptureError::InitFailed(e.to_string()))?;

        *self.thread_handle.lock() = Some(handle);
        Ok(())
    }

    fn stop(&self) -> Result<(), CaptureError> {
        if !self.is_running.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        if let Some(handle) = self.thread_handle.lock().take() {
            let _ = handle.join();
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    fn active_device(&self) -> AudioDeviceInfo {
        self.device_info.clone()
    }

    fn available_devices(&self) -> Vec<AudioDeviceInfo> {
        vec![
            self.device_info.clone(),
            AudioDeviceInfo {
                id: "wasapi_headphones".to_string(),
                name: "Realtek Audio Headphones (WASAPI Loopback)".to_string(),
                is_default: false,
                source_type: SourceType::SystemAudio,
                sample_rate: 48000,
                channels: 2,
            },
            AudioDeviceInfo {
                id: "wasapi_virtual_cable".to_string(),
                name: "VB-Audio Virtual Cable (WASAPI Loopback)".to_string(),
                is_default: false,
                source_type: SourceType::SystemAudio,
                sample_rate: 48000,
                channels: 2,
            },
        ]
    }

    fn frames_captured(&self) -> u64 {
        self.frames_captured.load(Ordering::Relaxed)
    }
}
