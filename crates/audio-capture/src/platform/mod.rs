use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformBackendType {
    WindowsWasapi,
    LinuxPipeWire,
    LinuxPulseAudio,
    MacOsScreenCaptureKit,
    MacOsCoreAudio,
    SyntheticFallback,
}

impl std::fmt::Display for PlatformBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformBackendType::WindowsWasapi => write!(f, "Windows WASAPI"),
            PlatformBackendType::LinuxPipeWire => write!(f, "Linux PipeWire"),
            PlatformBackendType::LinuxPulseAudio => write!(f, "Linux PulseAudio"),
            PlatformBackendType::MacOsScreenCaptureKit => write!(f, "macOS ScreenCaptureKit"),
            PlatformBackendType::MacOsCoreAudio => write!(f, "macOS CoreAudio"),
            PlatformBackendType::SyntheticFallback => write!(f, "Synthetic Audio Engine"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    PromptRequired,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformCapabilityInfo {
    pub backend_type: PlatformBackendType,
    pub backend_name: String,
    pub os_name: String,
    pub is_system_audio_supported: bool,
    pub is_process_audio_supported: bool,
    pub min_latency_ms: f32,
    pub permission_status: PermissionStatus,
    pub permission_description: String,
    pub requires_portal_approval: bool,
    pub requires_tcc_permission: bool,
}

impl PlatformCapabilityInfo {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                backend_type: PlatformBackendType::WindowsWasapi,
                backend_name: "Windows WASAPI Loopback".to_string(),
                os_name: "Windows".to_string(),
                is_system_audio_supported: true,
                is_process_audio_supported: true, // Windows 10 build 20348+ & Windows 11
                min_latency_ms: 10.0,
                permission_status: PermissionStatus::Granted,
                permission_description: "Standard audio loopback endpoint (no extra user prompt needed)".to_string(),
                requires_portal_approval: false,
                requires_tcc_permission: false,
            }
        }

        #[cfg(target_os = "linux")]
        {
            Self {
                backend_type: PlatformBackendType::LinuxPipeWire,
                backend_name: "Linux PipeWire Native (libpipewire-0.3)".to_string(),
                os_name: "Linux".to_string(),
                is_system_audio_supported: true,
                is_process_audio_supported: true, // PipeWire node / application target stream
                min_latency_ms: 8.0,
                permission_status: PermissionStatus::Granted,
                permission_description: "PipeWire sandbox portal token or native daemon socket".to_string(),
                requires_portal_approval: false,
                requires_tcc_permission: false,
            }
        }

        #[cfg(target_os = "macos")]
        {
            Self {
                backend_type: PlatformBackendType::MacOsScreenCaptureKit,
                backend_name: "macOS ScreenCaptureKit (SCStream)".to_string(),
                os_name: "macOS".to_string(),
                is_system_audio_supported: true, // macOS 13.0+
                is_process_audio_supported: true, // SCRunningApplication per-app audio stream
                min_latency_ms: 12.0,
                permission_status: PermissionStatus::Granted,
                permission_description: "Requires macOS Screen & System Audio Recording Permission (TCC)".to_string(),
                requires_portal_approval: false,
                requires_tcc_permission: true,
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Self {
                backend_type: PlatformBackendType::SyntheticFallback,
                backend_name: "Synthetic Loopback Audio Engine".to_string(),
                os_name: std::env::consts::OS.to_string(),
                is_system_audio_supported: true,
                is_process_audio_supported: true,
                min_latency_ms: 10.0,
                permission_status: PermissionStatus::NotApplicable,
                permission_description: "Synthetic test capture environment".to_string(),
                requires_portal_approval: false,
                requires_tcc_permission: false,
            }
        }
    }
}
