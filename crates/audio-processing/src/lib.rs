use audio_types::{AudioPcmFrame, AudioProfile};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid channel count: {0}")]
    InvalidChannels(u16),
    #[error("Sample rate must be positive, got {0}")]
    InvalidSampleRate(u32),
    #[error("Buffer length mismatch: expected {expected}, actual {actual}")]
    BufferMismatch { expected: usize, actual: usize },
}

/// Normalizes 32-bit floating-point audio samples (WASAPI default) into 16-bit signed PCM
pub fn float32_to_pcm16(input: &[f32], output: &mut [i16]) {
    assert!(output.len() >= input.len());
    for (i, &sample) in input.iter().enumerate() {
        let clamped = sample.clamp(-1.0, 1.0);
        output[i] = if clamped >= 0.0 {
            (clamped * 32767.0) as i16
        } else {
            (clamped * 32768.0) as i16
        };
    }
}

/// Converts interleaved mono samples into interleaved stereo samples
pub fn mono_to_stereo(input: &[i16], output: &mut [i16]) {
    assert!(output.len() >= input.len() * 2);
    for (i, &sample) in input.iter().enumerate() {
        output[i * 2] = sample;
        output[i * 2 + 1] = sample;
    }
}

/// Converts interleaved stereo samples into mono by averaging channels
pub fn stereo_to_mono(input: &[i16], output: &mut [i16]) {
    assert!(output.len() >= input.len() / 2);
    for i in 0..(input.len() / 2) {
        let left = input[i * 2] as i32;
        let right = input[i * 2 + 1] as i32;
        output[i] = ((left + right) / 2) as i16;
    }
}

/// Linear Fractional Resampler
/// Safely converts between sample rates (e.g. 44.1 kHz -> 48 kHz or 96 kHz -> 48 kHz)
pub struct Resampler {
    in_rate: u32,
    out_rate: u32,
    channels: u16,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32, channels: u16) -> Result<Self, ProcessingError> {
        if in_rate == 0 || out_rate == 0 {
            return Err(ProcessingError::InvalidSampleRate(in_rate.min(out_rate)));
        }
        if channels == 0 || channels > 8 {
            return Err(ProcessingError::InvalidChannels(channels));
        }
        Ok(Self {
            in_rate,
            out_rate,
            channels,
        })
    }

    pub fn resample(&self, input: &[i16]) -> Vec<i16> {
        if self.in_rate == self.out_rate {
            return input.to_vec();
        }

        let ch = self.channels as usize;
        let in_frames = input.len() / ch;
        let out_frames = ((in_frames as u64 * self.out_rate as u64) / self.in_rate as u64) as usize;
        let mut output = Vec::with_capacity(out_frames * ch);

        let ratio = self.in_rate as f64 / self.out_rate as f64;

        for out_frame in 0..out_frames {
            let src_idx_f = out_frame as f64 * ratio;
            let src_idx_floor = src_idx_f.floor() as usize;
            let frac = (src_idx_f - src_idx_floor as f64) as f32;

            let next_idx = (src_idx_floor + 1).min(in_frames - 1);

            for c in 0..ch {
                let s1 = input[src_idx_floor * ch + c] as f32;
                let s2 = input[next_idx * ch + c] as f32;
                let interpolated = s1 + frac * (s2 - s1);
                output.push(interpolated.clamp(-32768.0, 32767.0) as i16);
            }
        }

        output
    }
}

/// Frame Assembler with sequence counting and microsecond timestamping
pub struct FrameAssembler {
    profile: AudioProfile,
    buffer: Vec<i16>,
    sequence_number: u64,
}

impl FrameAssembler {
    pub fn new(profile: AudioProfile) -> Self {
        Self {
            profile,
            buffer: Vec::with_capacity(48000 * 2), // Pre-allocated 1 second headroom
            sequence_number: 0,
        }
    }

    pub fn push_samples(&mut self, samples: &[i16]) {
        self.buffer.extend_from_slice(samples);
    }

    /// Pulls complete frames of length `total_samples_per_frame`
    pub fn pop_frame(&mut self, current_time_us: u64) -> Option<AudioPcmFrame> {
        let frame_size = self.profile.total_samples_per_frame();
        if self.buffer.len() < frame_size {
            return None;
        }

        let frame_data: Vec<i16> = self.buffer.drain(0..frame_size).collect();
        let frame = AudioPcmFrame::new(
            self.sequence_number,
            current_time_us,
            self.profile.sample_rate,
            self.profile.channels,
            frame_data,
        );

        self.sequence_number += 1;
        Some(frame)
    }

    pub fn buffered_sample_count(&self) -> usize {
        self.buffer.len()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.sequence_number = 0;
    }
}

/// Bounded Ring Buffer for lockless/bounded audio callback passing
pub struct AudioRingBuffer {
    buffer: Vec<i16>,
    capacity: usize,
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0i16; capacity],
            capacity,
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    pub fn push_slice(&mut self, data: &[i16]) -> usize {
        let space = self.capacity - self.count;
        let to_write = data.len().min(space);

        for &val in &data[..to_write] {
            self.buffer[self.write_pos] = val;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
        self.count += to_write;
        to_write
    }

    pub fn pop_slice(&mut self, out: &mut [i16]) -> usize {
        let to_read = out.len().min(self.count);

        for val in out.iter_mut().take(to_read) {
            *val = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        self.count -= to_read;
        to_read
    }

    pub fn available_read(&self) -> usize {
        self.count
    }

    pub fn available_write(&self) -> usize {
        self.capacity - self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float32_to_pcm16_conversion() {
        let input = vec![-1.0f32, 0.0f32, 1.0f32, 0.5f32, -0.5f32];
        let mut output = vec![0i16; 5];
        float32_to_pcm16(&input, &mut output);

        assert_eq!(output[0], -32768);
        assert_eq!(output[1], 0);
        assert_eq!(output[2], 32767);
        assert_eq!(output[3], 16383);
        assert_eq!(output[4], -16384);
    }

    #[test]
    fn test_resampler_44100_to_48000() {
        let resampler = Resampler::new(44100, 48000, 2).expect("Resampler init failed");
        // 44100 frames at 2ch = 88200 samples
        let input = vec![1000i16; 4410 * 2]; // 0.1s audio
        let output = resampler.resample(&input);

        // 48000 * 0.1s * 2ch = 9600 samples
        assert_eq!(output.len(), 4800 * 2);
    }

    #[test]
    fn test_frame_assembler_chunking() {
        let profile = AudioProfile::default(); // 48kHz, 2ch, 20ms = 1920 samples
        let mut assembler = FrameAssembler::new(profile);

        let chunk1 = vec![42i16; 1000];
        let chunk2 = vec![42i16; 1000];
        assembler.push_samples(&chunk1);
        assert!(assembler.pop_frame(0).is_none());

        assembler.push_samples(&chunk2);
        let frame = assembler.pop_frame(20_000);
        assert!(frame.is_some());
        let f = frame.unwrap();
        assert_eq!(f.sequence_number, 0);
        assert_eq!(f.data.len(), 1920);
        assert_eq!(assembler.buffered_sample_count(), 80);
    }

    #[test]
    fn test_ring_buffer_fifo() {
        let mut rb = AudioRingBuffer::new(10);
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(rb.push_slice(&data), 5);
        assert_eq!(rb.available_read(), 5);

        let mut out = vec![0i16; 3];
        assert_eq!(rb.pop_slice(&mut out), 3);
        assert_eq!(out, vec![1, 2, 3]);
        assert_eq!(rb.available_read(), 2);
    }
}
