use audio_types::AudioProfile;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Jitter buffer entry holding encoded or raw audio payload
#[derive(Debug, Clone)]
pub struct BufferedPacket {
    pub sequence_number: u64,
    pub timestamp_us: u64,
    pub payload: Vec<u8>,
    pub received_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct JitterBufferStats {
    pub packets_received: u64,
    pub packets_played: u64,
    pub packets_dropped_late: u64,
    pub packets_lost_concealed: u64,
    pub underruns: u64,
    pub current_depth_frames: usize,
    pub estimated_jitter_ms: f32,
    pub current_latency_ms: f32,
}

pub enum PopResult {
    /// Packet ready for decoding and playback
    Packet(BufferedPacket),
    /// Packet missing; decoder should run PLC (Packet Loss Concealment)
    ConcealLost { sequence_number: u64 },
    /// Buffer underrun: no data available to play
    Underrun,
}

/// Dynamic adaptive jitter buffer
pub struct JitterBuffer {
    profile: AudioProfile,
    queue: BTreeMap<u64, BufferedPacket>,
    next_play_seq: Option<u64>,
    target_delay: Duration,
    min_delay: Duration,
    max_delay: Duration,
    initial_buffering: bool,
    start_time: Option<Instant>,
    stats: JitterBufferStats,
    last_transit_time_ms: Option<f32>,
}

impl JitterBuffer {
    pub fn new(profile: AudioProfile) -> Self {
        let (min_ms, target_ms, max_ms) = match profile.buffer_mode {
            audio_types::BufferMode::LowLatency => (15, 30, 80),
            audio_types::BufferMode::Adaptive => (30, 60, 150),
            audio_types::BufferMode::Conservative => (60, 120, 250),
        };

        Self {
            profile,
            queue: BTreeMap::new(),
            next_play_seq: None,
            target_delay: Duration::from_millis(target_ms),
            min_delay: Duration::from_millis(min_ms),
            max_delay: Duration::from_millis(max_ms),
            initial_buffering: true,
            start_time: None,
            stats: JitterBufferStats::default(),
            last_transit_time_ms: None,
        }
    }

    /// Pushes a received network packet into the buffer
    pub fn push(&mut self, seq: u64, timestamp_us: u64, payload: Vec<u8>) {
        self.stats.packets_received += 1;
        let now = Instant::now();

        // Calculate RFC 3550 style transit jitter if sender timestamp available
        let arrival_ms = (now.elapsed().as_micros() as f32) / 1000.0;
        let send_ms = (timestamp_us as f32) / 1000.0;
        let transit = arrival_ms - send_ms;

        if let Some(prev_transit) = self.last_transit_time_ms {
            let diff = (transit - prev_transit).abs();
            // Jitter smoothing filter: J = J + (|D| - J) / 16
            self.stats.estimated_jitter_ms += (diff - self.stats.estimated_jitter_ms) / 16.0;
        }
        self.last_transit_time_ms = Some(transit);

        // Discard packet if it is older than already played sequence
        if let Some(next_seq) = self.next_play_seq {
            if seq < next_seq {
                self.stats.packets_dropped_late += 1;
                return;
            }
        }

        self.queue.insert(
            seq,
            BufferedPacket {
                sequence_number: seq,
                timestamp_us,
                payload,
                received_at: now,
            },
        );

        if self.start_time.is_none() {
            self.start_time = Some(now);
        }

        // Check if initial buffering delay has elapsed
        if self.initial_buffering {
            let queued_duration_ms = (self.queue.len() as u32 * self.profile.frame_duration_ms) as u64;
            if queued_duration_ms >= self.target_delay.as_millis() as u64 {
                self.initial_buffering = false;
                if self.next_play_seq.is_none() {
                    self.next_play_seq = self.queue.keys().next().copied();
                }
            }
        }
    }

    /// Pops the next frame to feed into decoder and audio callback
    pub fn pop(&mut self) -> PopResult {
        if self.initial_buffering {
            return PopResult::Underrun;
        }

        let expected_seq = match self.next_play_seq {
            Some(seq) => seq,
            None => {
                if let Some(&first_seq) = self.queue.keys().next() {
                    self.next_play_seq = Some(first_seq);
                    first_seq
                } else {
                    self.stats.underruns += 1;
                    return PopResult::Underrun;
                }
            }
        };

        if let Some(packet) = self.queue.remove(&expected_seq) {
            self.next_play_seq = Some(expected_seq + 1);
            self.stats.packets_played += 1;
            self.update_depth_stats();
            PopResult::Packet(packet)
        } else {
            // Check if there are future packets in queue
            if let Some(&highest_seq) = self.queue.keys().last() {
                if highest_seq > expected_seq {
                    // Future packets arrived, so expected_seq was lost in transit
                    self.next_play_seq = Some(expected_seq + 1);
                    self.stats.packets_lost_concealed += 1;
                    self.update_depth_stats();
                    PopResult::ConcealLost {
                        sequence_number: expected_seq,
                    }
                } else {
                    self.stats.underruns += 1;
                    PopResult::Underrun
                }
            } else {
                self.stats.underruns += 1;
                PopResult::Underrun
            }
        }
    }

    fn update_depth_stats(&mut self) {
        self.stats.current_depth_frames = self.queue.len();
        self.stats.current_latency_ms = (self.queue.len() as f32) * (self.profile.frame_duration_ms as f32);

        // Adaptive adjustment of target delay based on jitter
        if self.profile.buffer_mode == audio_types::BufferMode::Adaptive {
            let recommended_ms = (self.stats.estimated_jitter_ms * 3.0).clamp(
                self.min_delay.as_millis() as f32,
                self.max_delay.as_millis() as f32,
            );
            self.target_delay = Duration::from_millis(recommended_ms as u64);
        }
    }

    pub fn stats(&self) -> JitterBufferStats {
        self.stats.clone()
    }

    pub fn reset(&mut self) {
        self.queue.clear();
        self.next_play_seq = None;
        self.initial_buffering = true;
        self.start_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_buffer_ordering_and_loss_concealment() {
        let profile = AudioProfile::default();
        let mut jb = JitterBuffer::new(profile);

        // Pre-fill packets 0, 1, 3 (packet 2 is dropped)
        jb.push(0, 0, vec![1, 2, 3]);
        jb.push(1, 20_000, vec![4, 5, 6]);
        jb.push(3, 60_000, vec![7, 8, 9]);

        // Force initial buffering end
        jb.initial_buffering = false;
        jb.next_play_seq = Some(0);

        // Pop seq 0
        match jb.pop() {
            PopResult::Packet(p) => assert_eq!(p.sequence_number, 0),
            _ => panic!("Expected packet 0"),
        }

        // Pop seq 1
        match jb.pop() {
            PopResult::Packet(p) => assert_eq!(p.sequence_number, 1),
            _ => panic!("Expected packet 1"),
        }

        // Pop seq 2 -> Should trigger ConcealLost because seq 3 is in queue
        match jb.pop() {
            PopResult::ConcealLost { sequence_number } => assert_eq!(sequence_number, 2),
            _ => panic!("Expected conceal lost for packet 2"),
        }

        // Pop seq 3
        match jb.pop() {
            PopResult::Packet(p) => assert_eq!(p.sequence_number, 3),
            _ => panic!("Expected packet 3"),
        }

        // Pop when queue is empty -> Underrun
        match jb.pop() {
            PopResult::Underrun => (),
            _ => panic!("Expected underrun"),
        }

        assert_eq!(jb.stats().packets_lost_concealed, 1);
        assert_eq!(jb.stats().packets_played, 3);
    }
}
