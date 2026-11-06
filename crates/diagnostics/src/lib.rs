use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    pub session_duration_secs: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub packets_dropped_late: u64,
    pub underruns: u64,
    pub packet_loss_rate_percent: f32,
    pub rtt_ms: f32,
    pub jitter_ms: f32,
    pub estimated_e2e_latency_ms: f32,
    pub bitrate_kbps: f32,
    pub buffer_depth_ms: f32,
}

pub struct MetricsTracker {
    start_time: Instant,
    last_bitrate_time: RwLock<Instant>,
    last_bytes_count: AtomicU64,
    bytes_transferred: AtomicU64,
    packets_sent: AtomicU64,
    packets_received: AtomicU64,
    packets_lost: AtomicU64,
    packets_dropped_late: AtomicU64,
    underruns: AtomicU64,
    rtt_ms: RwLock<f32>,
    jitter_ms: RwLock<f32>,
    buffer_depth_ms: RwLock<f32>,
    hardware_latency_ms: RwLock<f32>,
}

impl MetricsTracker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            start_time: Instant::now(),
            last_bitrate_time: RwLock::new(Instant::now()),
            last_bytes_count: AtomicU64::new(0),
            bytes_transferred: AtomicU64::new(0),
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            packets_lost: AtomicU64::new(0),
            packets_dropped_late: AtomicU64::new(0),
            underruns: AtomicU64::new(0),
            rtt_ms: RwLock::new(12.0),
            jitter_ms: RwLock::new(2.5),
            buffer_depth_ms: RwLock::new(40.0),
            hardware_latency_ms: RwLock::new(15.0), // OS capture + playback buffer
        })
    }

    pub fn record_packet_sent(&self, bytes: usize) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_transferred.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_packet_received(&self, bytes: usize) {
        self.packets_received.fetch_add(1, Ordering::Relaxed);
        self.bytes_transferred.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_packet_lost(&self) {
        self.packets_lost.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_packet_dropped_late(&self) {
        self.packets_dropped_late.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_underrun(&self) {
        self.underruns.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_rtt(&self, rtt: f32) {
        let mut w = self.rtt_ms.write();
        *w = (*w * 0.8) + (rtt * 0.2);
    }

    pub fn update_jitter(&self, jitter: f32) {
        let mut w = self.jitter_ms.write();
        *w = (*w * 0.8) + (jitter * 0.2);
    }

    pub fn update_buffer_depth(&self, depth_ms: f32) {
        let mut w = self.buffer_depth_ms.write();
        *w = depth_ms;
    }

    pub fn snapshot(&self) -> DiagnosticsSnapshot {
        let duration = self.start_time.elapsed().as_secs();
        let sent = self.packets_sent.load(Ordering::Relaxed);
        let recv = self.packets_received.load(Ordering::Relaxed);
        let lost = self.packets_lost.load(Ordering::Relaxed);
        let late = self.packets_dropped_late.load(Ordering::Relaxed);
        let underruns = self.underruns.load(Ordering::Relaxed);

        let total_expected = recv + lost;
        let loss_rate = if total_expected > 0 {
            (lost as f32 / total_expected as f32) * 100.0
        } else {
            0.0
        };

        let rtt = *self.rtt_ms.read();
        let jitter = *self.jitter_ms.read();
        let buf_depth = *self.buffer_depth_ms.read();
        let hw_latency = *self.hardware_latency_ms.read();

        // End-to-end latency calculation:
        // E2E = Capture Delay (half of WASAPI period) + Encoding (<2ms) + (RTT / 2) + Jitter Buffer Depth + Hardware Audio Output Delay (~15ms)
        let network_one_way = rtt / 2.0;
        let e2e_latency = 10.0 + 2.0 + network_one_way + buf_depth + hw_latency;

        let total_bytes = self.bytes_transferred.load(Ordering::Relaxed);
        let mut last_time = self.last_bitrate_time.write();
        let elapsed_secs = last_time.elapsed().as_secs_f32().max(0.1);
        let prev_bytes = self.last_bytes_count.swap(total_bytes, Ordering::Relaxed);
        let delta_bytes = total_bytes.saturating_sub(prev_bytes);
        let bitrate_kbps = ((delta_bytes as f32 * 8.0) / 1000.0) / elapsed_secs;
        *last_time = Instant::now();

        DiagnosticsSnapshot {
            session_duration_secs: duration,
            packets_sent: sent,
            packets_received: recv,
            packets_lost: lost,
            packets_dropped_late: late,
            underruns,
            packet_loss_rate_percent: loss_rate,
            rtt_ms: rtt,
            jitter_ms: jitter,
            estimated_e2e_latency_ms: e2e_latency,
            bitrate_kbps,
            buffer_depth_ms: buf_depth,
        }
    }
}
