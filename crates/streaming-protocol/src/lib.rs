use audio_types::{AudioProfile, AudioSourceInfo, CodecType};
use byteorder::{BigEndian, ByteOrder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u8 = 1;
pub const AUDIO_MAGIC: &[u8; 4] = b"ABRG"; // AudioBridge datagram magic
pub const AUDIO_HEADER_SIZE: usize = 30;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid magic bytes in datagram")]
    InvalidMagic,
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),
    #[error("Buffer underflow: expected at least {expected} bytes, got {actual}")]
    BufferUnderflow { expected: usize, actual: usize },
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Payload length mismatch: declared {declared}, actual {actual}")]
    PayloadLengthMismatch { declared: usize, actual: usize },
}

/// Binary Audio Datagram header representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDatagramHeader {
    pub version: u8,
    pub flags: u8,
    pub codec: CodecType,
    pub stream_id: u32,
    pub sequence_number: u64,
    pub timestamp_us: u64,
    pub payload_len: u16,
}

impl AudioDatagramHeader {
    pub fn encode(&self, out: &mut [u8]) -> Result<(), ProtocolError> {
        if out.len() < AUDIO_HEADER_SIZE {
            return Err(ProtocolError::BufferUnderflow {
                expected: AUDIO_HEADER_SIZE,
                actual: out.len(),
            });
        }

        out[0..4].copy_from_slice(AUDIO_MAGIC);
        out[4] = self.version;
        out[5] = self.flags;
        out[6] = match self.codec {
            CodecType::Opus => 1,
            CodecType::Pcm => 2,
        };
        out[7] = 0; // reserved
        BigEndian::write_u32(&mut out[8..12], self.stream_id);
        BigEndian::write_u64(&mut out[12..20], self.sequence_number);
        BigEndian::write_u64(&mut out[20..28], self.timestamp_us);
        BigEndian::write_u16(&mut out[28..30], self.payload_len);

        Ok(())
    }

    pub fn decode(src: &[u8]) -> Result<Self, ProtocolError> {
        if src.len() < AUDIO_HEADER_SIZE {
            return Err(ProtocolError::BufferUnderflow {
                expected: AUDIO_HEADER_SIZE,
                actual: src.len(),
            });
        }

        if &src[0..4] != AUDIO_MAGIC {
            return Err(ProtocolError::InvalidMagic);
        }

        let version = src[4];
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        let flags = src[5];
        let codec = match src[6] {
            1 => CodecType::Opus,
            2 => CodecType::Pcm,
            _ => CodecType::Opus,
        };

        let stream_id = BigEndian::read_u32(&src[8..12]);
        let sequence_number = BigEndian::read_u64(&src[12..20]);
        let timestamp_us = BigEndian::read_u64(&src[20..28]);
        let payload_len = BigEndian::read_u16(&src[28..30]);

        Ok(Self {
            version,
            flags,
            codec,
            stream_id,
            sequence_number,
            timestamp_us,
            payload_len,
        })
    }
}

/// Control plane messages exchanged over reliable channel (TCP/QUIC Stream)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum ControlMessage {
    // Handshake
    Hello {
        client_id: String,
        client_name: String,
        client_version: String,
        protocol_version: u8,
    },
    HelloAck {
        server_name: String,
        server_version: String,
        server_id: String,
        requires_pairing: bool,
    },

    // Authentication for already-trusted devices
    Authenticate {
        client_id: String,
        auth_token: String,
    },
    AuthResponse {
        success: bool,
        session_id: Option<u32>,
        error_message: Option<String>,
    },

    // Pairing (New unauthenticated devices)
    PairRequest {
        client_id: String,
        client_name: String,
    },
    PairChallenge {
        challenge_id: String,
        pairing_code: String,
        expires_in_secs: u32,
    },
    PairVerify {
        challenge_id: String,
        pairing_code: String,
    },
    PairResponse {
        success: bool,
        auth_token: Option<String>,
        error_message: Option<String>,
    },

    // Trusted Devices Management
    RevokeDevice {
        client_id: String,
    },
    DeviceRevoked {
        client_id: String,
        success: bool,
    },

    // Source Selection
    GetSources,
    SourceList {
        sources: Vec<AudioSourceInfo>,
    },
    SelectSource {
        source_id: String,
    },
    SourceChanged {
        source_id: String,
        fallback: bool,
        message: String,
    },

    // Stream negotiation
    SetProfile {
        profile: AudioProfile,
    },
    ProfileChanged {
        profile: AudioProfile,
    },
    SetDeviceMute {
        is_muted: bool,
        volume: f32,
    },
    DeviceMuteChanged {
        is_muted: bool,
        volume: f32,
    },
    GetSessions,
    SessionList {
        sessions: Vec<serde_json::Value>,
    },
    StartStream {
        media_port: u16,
    },
    StreamStarted {
        stream_id: u32,
        session_id: u32,
        assigned_port: u16,
        profile: AudioProfile,
    },
    StopStream,
    StreamStopped {
        reason: String,
    },

    // Keepalive and latency diagnostics
    Ping {
        client_send_time_us: u64,
    },
    Pong {
        client_send_time_us: u64,
        server_recv_time_us: u64,
    },
    Disconnect {
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_datagram_header_serialization() {
        let header = AudioDatagramHeader {
            version: PROTOCOL_VERSION,
            flags: 0x01,
            codec: CodecType::Opus,
            stream_id: 42,
            sequence_number: 1001,
            timestamp_us: 1729000000,
            payload_len: 128,
        };

        let mut buf = vec![0u8; AUDIO_HEADER_SIZE + 1];
        header.encode(&mut buf).expect("Encode failed");

        let decoded = AudioDatagramHeader::decode(&buf).expect("Decode failed");
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_control_message_roundtrip() {
        let msg = ControlMessage::Hello {
            client_id: "android-pixel9-123".to_string(),
            client_name: "Pixel 9 Pro".to_string(),
            client_version: "1.0.0".to_string(),
            protocol_version: PROTOCOL_VERSION,
        };

        let serialized = serde_json::to_string(&msg).expect("Serialize failed");
        let deserialized: ControlMessage = serde_json::from_str(&serialized).expect("Deserialize failed");
        assert_eq!(msg, deserialized);
    }
}
