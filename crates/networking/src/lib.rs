pub mod connection_manager;
pub mod control_server;
pub mod discovery;
pub mod pairing;
pub mod session;

pub use connection_manager::{
    AdbDeviceStatus, ConnectionManager, ConnectionPathTelemetry, ConnectionTransportMode,
    NetworkInterfaceInfo, TransportPriorityPolicy, UsbTetheringStatus,
};
pub use control_server::{ControlPlaneServer, ServerConfig};
pub use discovery::{DiscoveredHost, ServiceMetadata, DEFAULT_CONTROL_PORT, DEFAULT_MEDIA_PORT, MDNS_SERVICE_TYPE};
pub use pairing::{generate_secure_auth_token, PairingChallengeInfo, PairingError, PairingManager, MAX_PAIRING_ATTEMPTS, PAIRING_EXPIRY_SECS};
pub use session::{ActiveSession, SessionInfo, SessionManager};

use audio_types::CodecType;
use diagnostics::MetricsTracker;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use streaming_protocol::{
    AudioDatagramHeader, AUDIO_HEADER_SIZE, PROTOCOL_VERSION,
};
use thiserror::Error;
use tokio::net::UdpSocket;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Protocol error: {0}")]
    Protocol(#[from] streaming_protocol::ProtocolError),
    #[error("Socket closed")]
    Closed,
    #[error("Connection timeout")]
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Streaming,
    Reconnecting,
}

/// UDP Datagram audio packet sender (media plane)
pub struct AudioDatagramSender {
    socket: Arc<UdpSocket>,
    target_addr: SocketAddr,
    stream_id: u32,
    seq: u64,
    codec: CodecType,
    metrics: Arc<MetricsTracker>,
    buffer: Vec<u8>,
}

impl AudioDatagramSender {
    pub async fn bind(local_addr: SocketAddr, target_addr: SocketAddr, stream_id: u32, codec: CodecType, metrics: Arc<MetricsTracker>) -> Result<Self, NetworkError> {
        let socket = UdpSocket::bind(local_addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
            target_addr,
            stream_id,
            seq: 0,
            codec,
            metrics,
            buffer: vec![0u8; 1500], // Standard MTU buffer
        })
    }

    pub async fn send_frame(&mut self, payload: &[u8], timestamp_us: u64) -> Result<(), NetworkError> {
        let total_size = AUDIO_HEADER_SIZE + payload.len();
        if self.buffer.len() < total_size {
            self.buffer.resize(total_size, 0);
        }

        let header = AudioDatagramHeader {
            version: PROTOCOL_VERSION,
            flags: 0,
            codec: self.codec,
            stream_id: self.stream_id,
            sequence_number: self.seq,
            timestamp_us,
            payload_len: payload.len() as u16,
        };

        header.encode(&mut self.buffer[..AUDIO_HEADER_SIZE])?;
        self.buffer[AUDIO_HEADER_SIZE..total_size].copy_from_slice(payload);

        self.socket.send_to(&self.buffer[..total_size], self.target_addr).await?;
        self.metrics.record_packet_sent(total_size);
        self.seq += 1;

        Ok(())
    }
}

/// Incoming audio frame packet received over network
#[derive(Debug, Clone)]
pub struct ReceivedAudioPacket {
    pub header: AudioDatagramHeader,
    pub payload: Vec<u8>,
}

/// UDP Datagram audio packet receiver (media plane)
pub struct AudioDatagramReceiver {
    socket: Arc<UdpSocket>,
    metrics: Arc<MetricsTracker>,
}

impl AudioDatagramReceiver {
    pub async fn bind(listen_addr: SocketAddr, metrics: Arc<MetricsTracker>) -> Result<Self, NetworkError> {
        let socket = UdpSocket::bind(listen_addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
            metrics,
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, NetworkError> {
        Ok(self.socket.local_addr()?)
    }

    pub async fn receive_packet(&self) -> Result<ReceivedAudioPacket, NetworkError> {
        let mut buf = vec![0u8; 2048];
        let (len, _sender) = self.socket.recv_from(&mut buf).await?;
        if len < AUDIO_HEADER_SIZE {
            return Err(NetworkError::Protocol(
                streaming_protocol::ProtocolError::BufferUnderflow {
                    expected: AUDIO_HEADER_SIZE,
                    actual: len,
                },
            ));
        }

        let header = AudioDatagramHeader::decode(&buf[..AUDIO_HEADER_SIZE])?;
        let payload = buf[AUDIO_HEADER_SIZE..len].to_vec();

        self.metrics.record_packet_received(len);

        Ok(ReceivedAudioPacket { header, payload })
    }
}

/// Reconnection policy manager with exponential backoff
#[derive(Debug, Clone)]
pub struct ReconnectionPolicy {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_retries: usize,
    pub current_retry: usize,
}

impl Default for ReconnectionPolicy {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(8),
            max_retries: 10,
            current_retry: 0,
        }
    }
}

impl ReconnectionPolicy {
    pub fn next_backoff(&mut self) -> Option<Duration> {
        if self.current_retry >= self.max_retries {
            return None;
        }
        let factor = 2u32.saturating_pow(self.current_retry as u32);
        let backoff = (self.base_delay * factor).min(self.max_delay);
        self.current_retry += 1;
        Some(backoff)
    }

    pub fn reset(&mut self) {
        self.current_retry = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_audio_transport_send_receive() {
        let metrics = MetricsTracker::new();
        let receiver = AudioDatagramReceiver::bind("127.0.0.1:0".parse().unwrap(), metrics.clone())
            .await
            .expect("Failed to bind receiver");
        let listen_addr = receiver.local_addr().unwrap();

        let mut sender = AudioDatagramSender::bind(
            "127.0.0.1:0".parse().unwrap(),
            listen_addr,
            1,
            CodecType::Opus,
            metrics.clone(),
        )
        .await
        .expect("Failed to bind sender");

        let test_payload = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];
        sender
            .send_frame(&test_payload, 500_000)
            .await
            .expect("Send failed");

        let received = receiver.receive_packet().await.expect("Receive failed");
        assert_eq!(received.header.sequence_number, 0);
        assert_eq!(received.header.timestamp_us, 500_000);
        assert_eq!(received.payload, test_payload);
    }

    #[test]
    fn test_reconnection_exponential_backoff() {
        let mut policy = ReconnectionPolicy::default();
        let d1 = policy.next_backoff().unwrap();
        let d2 = policy.next_backoff().unwrap();
        let d3 = policy.next_backoff().unwrap();

        assert_eq!(d1, Duration::from_millis(500));
        assert_eq!(d2, Duration::from_millis(1000));
        assert_eq!(d3, Duration::from_millis(2000));

        policy.reset();
        assert_eq!(policy.current_retry, 0);
    }
}
