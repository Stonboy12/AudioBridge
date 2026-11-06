use audio_types::{AudioProfile, BufferMode, CodecType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Configuration validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSettings {
    pub codec: String, // "opus" or "pcm"
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate_kbps: u32,
    pub frame_duration_ms: u32,
    pub buffer_mode: String, // "low_latency", "adaptive", "conservative"
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            codec: "opus".to_string(),
            sample_rate: 48000,
            channels: 2,
            bitrate_kbps: 128,
            frame_duration_ms: 20,
            buffer_mode: "adaptive".to_string(),
        }
    }
}

impl AudioSettings {
    pub fn to_audio_profile(&self) -> AudioProfile {
        let codec = match self.codec.to_lowercase().as_str() {
            "pcm" => CodecType::Pcm,
            _ => CodecType::Opus,
        };
        let buffer_mode = match self.buffer_mode.to_lowercase().as_str() {
            "low_latency" => BufferMode::LowLatency,
            "conservative" => BufferMode::Conservative,
            _ => BufferMode::Adaptive,
        };

        AudioProfile {
            profile_id: format!("{}-{}-{}k", self.codec, self.channels, self.bitrate_kbps),
            codec,
            sample_rate: self.sample_rate,
            channels: self.channels,
            bitrate_kbps: self.bitrate_kbps,
            frame_duration_ms: self.frame_duration_ms,
            buffer_mode,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSettings {
    pub control_port: u16,
    pub media_port: u16,
    pub discovery_enabled: bool,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            control_port: 40120,
            media_port: 40121,
            discovery_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralSettings {
    pub auto_start_capture: bool,
    pub log_level: String,
    pub theme: String,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_start_capture: false,
            log_level: "info".to_string(),
            theme: "dark".to_string(),
        }
    }
}

/// Represents a securely paired and trusted mobile receiver device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustedDevice {
    pub client_id: String,
    pub client_name: String,
    pub auth_token: String,
    pub paired_at: String,
    pub last_seen: String,
}

/// Root Application Configuration Structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub schema_version: u32,
    pub audio: AudioSettings,
    pub network: NetworkSettings,
    pub general: GeneralSettings,
    #[serde(default)]
    pub trusted_devices: Vec<TrustedDevice>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            audio: AudioSettings::default(),
            network: NetworkSettings::default(),
            general: GeneralSettings::default(),
            trusted_devices: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn add_trusted_device(&mut self, device: TrustedDevice) {
        self.trusted_devices.retain(|d| d.client_id != device.client_id);
        self.trusted_devices.push(device);
    }

    pub fn remove_trusted_device(&mut self, client_id: &str) -> bool {
        let initial_len = self.trusted_devices.len();
        self.trusted_devices.retain(|d| d.client_id != client_id);
        self.trusted_devices.len() < initial_len
    }

    pub fn is_trusted(&self, client_id: &str, token: &str) -> bool {
        self.trusted_devices.iter().any(|d| d.client_id == client_id && d.auth_token == token)
    }

    pub fn get_trusted_device(&self, client_id: &str) -> Option<&TrustedDevice> {
        self.trusted_devices.iter().find(|d| d.client_id == client_id)
    }

    pub fn update_last_seen(&mut self, client_id: &str, timestamp: &str) {
        if let Some(dev) = self.trusted_devices.iter_mut().find(|d| d.client_id == client_id) {
            dev.last_seen = timestamp.to_string();
        }
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ConfigError::Validation(format!(
                "Unsupported schema version: expected {}, got {}",
                CURRENT_SCHEMA_VERSION, self.schema_version
            )));
        }

        if self.audio.sample_rate != 44100 && self.audio.sample_rate != 48000 && self.audio.sample_rate != 96000 {
            return Err(ConfigError::Validation(format!(
                "Invalid sample rate: {}",
                self.audio.sample_rate
            )));
        }

        if self.audio.channels != 1 && self.audio.channels != 2 {
            return Err(ConfigError::Validation(format!(
                "Invalid channels: {}",
                self.audio.channels
            )));
        }

        if self.audio.bitrate_kbps < 32 || self.audio.bitrate_kbps > 512 {
            return Err(ConfigError::Validation(format!(
                "Bitrate out of bounds [32-512]: {}",
                self.audio.bitrate_kbps
            )));
        }

        if self.audio.frame_duration_ms != 10 && self.audio.frame_duration_ms != 20 {
            return Err(ConfigError::Validation(format!(
                "Unsupported frame duration: {} ms (must be 10 or 20)",
                self.audio.frame_duration_ms
            )));
        }

        if self.network.control_port < 1024 || self.network.media_port < 1024 {
            return Err(ConfigError::Validation("Ports must be in unprivileged range (1024-65535)".to_string()));
        }

        Ok(())
    }

    /// Atomically writes configuration to disk via tempfile to prevent corruption
    pub fn save_atomic<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        self.validate()?;
        let path = path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;

        let toml_str = toml::to_string_pretty(self)?;
        let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
        temp_file.write_all(toml_str.as_bytes())?;
        temp_file.flush()?;

        // Atomic persist/rename
        temp_file.persist(path).map_err(|e| ConfigError::Io(e.error))?;
        Ok(())
    }

    /// Loads configuration from file with fallback to default
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    if config.validate().is_ok() {
                        config
                    } else {
                        Self::default()
                    }
                }
                Err(_) => Self::default(),
            },
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_atomic_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("audiobridge.toml");

        let mut config = AppConfig::default();
        config.audio.bitrate_kbps = 192;
        config.network.control_port = 45000;

        config.save_atomic(&config_path).expect("Failed to save config atomically");
        assert!(config_path.exists());

        let loaded = AppConfig::load_or_default(&config_path);
        assert_eq!(loaded.audio.bitrate_kbps, 192);
        assert_eq!(loaded.network.control_port, 45000);
    }

    #[test]
    fn test_config_validation_rejection() {
        let mut invalid = AppConfig::default();
        invalid.audio.bitrate_kbps = 1000; // invalid > 512
        assert!(invalid.validate().is_err());
    }
}
