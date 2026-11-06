use serde::{Deserialize, Serialize};

/// Audio sample representation format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    I16,
    F32,
}

/// Supported audio compression and transport codecs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodecType {
    /// Opus low-latency interactive audio codec (RFC 6716)
    Opus,
    /// Uncompressed Linear PCM (for diagnostic / compatibility / fallback)
    Pcm,
}

/// Buffer latency management mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BufferMode {
    /// Lowest latency possible (target ~20-40ms, vulnerable to severe network jitter)
    LowLatency,
    /// Dynamically expands and contracts with measured jitter (target ~50-80ms)
    Adaptive,
    /// Extra headroom for unstable Wi-Fi connections (target ~100-150ms)
    Conservative,
}

/// Audio profile negotiated between desktop sender and listener
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioProfile {
    pub profile_id: String,
    pub codec: CodecType,
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate_kbps: u32,
    pub frame_duration_ms: u32,
    pub buffer_mode: BufferMode,
}

impl Default for AudioProfile {
    fn default() -> Self {
        Self {
            profile_id: "default-opus-stereo".to_string(),
            codec: CodecType::Opus,
            sample_rate: 48000,
            channels: 2,
            bitrate_kbps: 128,
            frame_duration_ms: 20,
            buffer_mode: BufferMode::Adaptive,
        }
    }
}

impl AudioProfile {
    /// Samples per channel per frame (e.g. 48000 * 20 / 1000 = 960)
    pub fn samples_per_frame(&self) -> usize {
        ((self.sample_rate as u64 * self.frame_duration_ms as u64) / 1000) as usize
    }

    /// Total interleaved samples per frame (samples_per_frame * channels)
    pub fn total_samples_per_frame(&self) -> usize {
        self.samples_per_frame() * self.channels as usize
    }
}

/// An audio frame representing a discrete packet of PCM samples
#[derive(Debug, Clone, PartialEq)]
pub struct AudioPcmFrame {
    pub sequence_number: u64,
    pub timestamp_us: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<i16>,
}

impl AudioPcmFrame {
    pub fn new(sequence_number: u64, timestamp_us: u64, sample_rate: u32, channels: u16, data: Vec<i16>) -> Self {
        Self {
            sequence_number,
            timestamp_us,
            sample_rate,
            channels,
            data,
        }
    }
}

/// Audio source metadata descriptor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    SystemAudio,
    ProcessAudio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioSourceInfo {
    pub source_id: String,
    pub display_name: String,
    pub source_type: SourceType,
    #[serde(default)]
    pub process_id: Option<u32>,
    #[serde(default)]
    pub process_name: Option<String>,
    pub is_available: bool,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub listener_count: usize,
    pub channels: u16,
    pub sample_rate: u32,
}

impl AudioSourceInfo {
    pub fn new_system(source_id: &str, display_name: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            display_name: display_name.to_string(),
            source_type: SourceType::SystemAudio,
            process_id: None,
            process_name: None,
            is_available: true,
            is_active: false,
            listener_count: 0,
            channels: 2,
            sample_rate: 48000,
        }
    }

    pub fn new_process(source_id: &str, display_name: &str, pid: u32, proc_name: &str) -> Self {
        Self {
            source_id: source_id.to_string(),
            display_name: display_name.to_string(),
            source_type: SourceType::ProcessAudio,
            process_id: Some(pid),
            process_name: Some(proc_name.to_string()),
            is_available: true,
            is_active: false,
            listener_count: 0,
            channels: 2,
            sample_rate: 48000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_profile_calculation() {
        let profile = AudioProfile::default();
        assert_eq!(profile.samples_per_frame(), 960);
        assert_eq!(profile.total_samples_per_frame(), 1920);
    }
}
