pub mod platform;

use audio_types::SourceType;
use parking_lot::Mutex;
pub use platform::{PermissionStatus, PlatformBackendType, PlatformCapabilityInfo};
use platform::linux::LinuxAudioCaptureEngine;
use platform::macos::MacOsAudioCaptureEngine;
use platform::windows::WindowsAudioCaptureEngine;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Failed to initialize capture client: {0}")]
    InitFailed(String),
    #[error("Device invalidated or disconnected")]
    DeviceInvalidated,
    #[error("Capture already running")]
    AlreadyRunning,
    #[error("Capture not running")]
    NotRunning,
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub source_type: SourceType,
    pub sample_rate: u32,
    pub channels: u16,
}

pub type AudioSampleCallback = Arc<dyn Fn(&[f32], usize) + Send + Sync>;

/// Abstraction for OS audio capture engine (WASAPI / PipeWire / ScreenCaptureKit / Synthetic)
pub trait AudioCaptureEngine: Send + Sync {
    fn start(&self, callback: AudioSampleCallback) -> Result<(), CaptureError>;
    fn stop(&self) -> Result<(), CaptureError>;
    fn is_running(&self) -> bool;
    fn active_device(&self) -> AudioDeviceInfo;
    fn available_devices(&self) -> Vec<AudioDeviceInfo>;
    fn frames_captured(&self) -> u64;
}

/// Factory function to create the native platform capture engine based on target OS
pub fn create_platform_capture_engine(sample_rate: u32, channels: u16) -> Arc<dyn AudioCaptureEngine> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(WindowsAudioCaptureEngine::new(sample_rate, channels))
    }

    #[cfg(target_os = "linux")]
    {
        Arc::new(LinuxAudioCaptureEngine::new(sample_rate, channels))
    }

    #[cfg(target_os = "macos")]
    {
        Arc::new(MacOsAudioCaptureEngine::new(sample_rate, channels))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Arc::new(SyntheticLoopbackCapture::new(sample_rate, channels))
    }
}

/// Fallback / Synthetic Capture implementation
pub struct SyntheticLoopbackCapture {
    is_running: Arc<AtomicBool>,
    device_info: AudioDeviceInfo,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    frames_captured: Arc<AtomicU64>,
}

impl SyntheticLoopbackCapture {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            device_info: AudioDeviceInfo {
                id: "synthetic_default_render".to_string(),
                name: "Synthetic Loopback Audio Device (Diagnostic Engine)".to_string(),
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

impl AudioCaptureEngine for SyntheticLoopbackCapture {
    fn start(&self, callback: AudioSampleCallback) -> Result<(), CaptureError> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return Err(CaptureError::AlreadyRunning);
        }

        let is_running = self.is_running.clone();
        let sr = self.device_info.sample_rate;
        let ch = self.device_info.channels as usize;
        let counter = self.frames_captured.clone();

        let handle = thread::Builder::new()
            .name("AudioBridge_Capture_Thread".to_string())
            .spawn(move || {
                let frame_samples = (sr as f32 * 0.01) as usize; // 10ms frame period
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
                id: "synthetic_aux_device".to_string(),
                name: "Synthetic Secondary Audio Sink (Virtual Loopback)".to_string(),
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

pub use platform::linux::LinuxAudioCaptureEngine as LinuxCapture;
pub use platform::macos::MacOsAudioCaptureEngine as MacOsCapture;
pub use platform::windows::WindowsAudioCaptureEngine as WindowsCapture;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_platform_capability_detection() {
        let cap = PlatformCapabilityInfo::current();
        assert!(cap.is_system_audio_supported);
        assert!(cap.min_latency_ms <= 15.0);
    }

    #[test]
    fn test_windows_capture_engine_lifecycle() {
        let engine = WindowsAudioCaptureEngine::new(48000, 2);
        assert!(!engine.is_running());
        let count = Arc::new(AtomicUsize::new(0));
        let c_clone = count.clone();
        engine.start(Arc::new(move |samples, _| {
            c_clone.fetch_add(samples.len(), Ordering::Relaxed);
        })).unwrap();
        assert!(engine.is_running());
        thread::sleep(Duration::from_millis(30));
        engine.stop().unwrap();
        assert!(!engine.is_running());
        assert!(count.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_linux_capture_engine_lifecycle() {
        let engine = LinuxAudioCaptureEngine::new(48000, 2);
        assert!(!engine.is_running());
        let count = Arc::new(AtomicUsize::new(0));
        let c_clone = count.clone();
        engine.start(Arc::new(move |samples, _| {
            c_clone.fetch_add(samples.len(), Ordering::Relaxed);
        })).unwrap();
        assert!(engine.is_running());
        thread::sleep(Duration::from_millis(30));
        engine.stop().unwrap();
        assert!(!engine.is_running());
        assert!(count.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_macos_capture_engine_lifecycle() {
        let engine = MacOsAudioCaptureEngine::new(48000, 2);
        assert!(!engine.is_running());
        let count = Arc::new(AtomicUsize::new(0));
        let c_clone = count.clone();
        engine.start(Arc::new(move |samples, _| {
            c_clone.fetch_add(samples.len(), Ordering::Relaxed);
        })).unwrap();
        assert!(engine.is_running());
        thread::sleep(Duration::from_millis(30));
        engine.stop().unwrap();
        assert!(!engine.is_running());
        assert!(count.load(Ordering::Relaxed) > 0);
    }
}
