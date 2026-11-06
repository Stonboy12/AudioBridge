use audio_types::{AudioProfile, CodecType};
use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Opus encoder error: {0}")]
    Opus(#[from] opus::Error),
    #[error("Invalid buffer size or sample count: expected {expected}, got {actual}")]
    BufferSizeMismatch { expected: usize, actual: usize },
    #[error("Unsupported configuration: {0}")]
    UnsupportedConfig(String),
}

/// Generic interface for audio encoders
pub trait AudioEncoder: Send {
    /// Encodes interleaved PCM i16 samples into compressed bytes
    fn encode(&mut self, pcm: &[i16], output: &mut [u8]) -> Result<usize, CodecError>;
    /// Returns the codec type
    fn codec_type(&self) -> CodecType;
}

/// Generic interface for audio decoders
pub trait AudioDecoder: Send {
    /// Decodes compressed bytes into interleaved PCM i16 samples
    fn decode(&mut self, packet: Option<&[u8]>, output: &mut [i16], fec: bool) -> Result<usize, CodecError>;
    /// Returns the codec type
    fn codec_type(&self) -> CodecType;
}

/// Opus Encoder implementation
pub struct OpusAudioEncoder {
    encoder: opus::Encoder,
    profile: AudioProfile,
}

impl OpusAudioEncoder {
    pub fn new(profile: &AudioProfile) -> Result<Self, CodecError> {
        let channels = match profile.channels {
            1 => opus::Channels::Mono,
            2 => opus::Channels::Stereo,
            other => return Err(CodecError::UnsupportedConfig(format!("Unsupported channels: {other}"))),
        };

        let mut encoder = opus::Encoder::new(
            profile.sample_rate,
            channels,
            opus::Application::LowDelay,
        )?;

        encoder.set_bitrate(opus::Bitrate::Bits(profile.bitrate_kbps as i32 * 1000))?;
        // Enable Forward Error Correction (FEC) for resilience on Wi-Fi
        encoder.set_inband_fec(true)?;
        // Set expected packet loss estimate to 5% for FEC tuning
        encoder.set_packet_loss_perc(5)?;

        Ok(Self {
            encoder,
            profile: profile.clone(),
        })
    }
}

impl AudioEncoder for OpusAudioEncoder {
    fn encode(&mut self, pcm: &[i16], output: &mut [u8]) -> Result<usize, CodecError> {
        let expected_samples = self.profile.total_samples_per_frame();
        if pcm.len() != expected_samples {
            return Err(CodecError::BufferSizeMismatch {
                expected: expected_samples,
                actual: pcm.len(),
            });
        }
        let bytes_written = self.encoder.encode(pcm, output)?;
        Ok(bytes_written)
    }

    fn codec_type(&self) -> CodecType {
        CodecType::Opus
    }
}

/// Opus Decoder implementation
pub struct OpusAudioDecoder {
    decoder: opus::Decoder,
    profile: AudioProfile,
}

impl OpusAudioDecoder {
    pub fn new(profile: &AudioProfile) -> Result<Self, CodecError> {
        let channels = match profile.channels {
            1 => opus::Channels::Mono,
            2 => opus::Channels::Stereo,
            other => return Err(CodecError::UnsupportedConfig(format!("Unsupported channels: {other}"))),
        };

        let decoder = opus::Decoder::new(profile.sample_rate, channels)?;

        Ok(Self {
            decoder,
            profile: profile.clone(),
        })
    }
}

impl AudioDecoder for OpusAudioDecoder {
    fn decode(&mut self, packet: Option<&[u8]>, output: &mut [i16], fec: bool) -> Result<usize, CodecError> {
        let _samples_per_channel = self.profile.samples_per_frame();
        let samples_decoded = match packet {
            Some(data) => self.decoder.decode(data, output, fec)?,
            None => {
                // Packet loss concealment (PLC)
                self.decoder.decode(&[], output, false)?
            }
        };
        Ok(samples_decoded)
    }

    fn codec_type(&self) -> CodecType {
        CodecType::Opus
    }
}

/// Linear PCM Encoder (identity uncompressed transport)
pub struct PcmAudioEncoder {
    profile: AudioProfile,
}

impl PcmAudioEncoder {
    pub fn new(profile: &AudioProfile) -> Self {
        Self {
            profile: profile.clone(),
        }
    }
}

impl AudioEncoder for PcmAudioEncoder {
    fn encode(&mut self, pcm: &[i16], output: &mut [u8]) -> Result<usize, CodecError> {
        let expected_samples = self.profile.total_samples_per_frame();
        if pcm.len() != expected_samples {
            return Err(CodecError::BufferSizeMismatch {
                expected: expected_samples,
                actual: pcm.len(),
            });
        }
        let needed_bytes = pcm.len() * 2;
        if output.len() < needed_bytes {
            return Err(CodecError::BufferSizeMismatch {
                expected: needed_bytes,
                actual: output.len(),
            });
        }
        for (i, &sample) in pcm.iter().enumerate() {
            LittleEndian::write_i16(&mut output[i * 2..i * 2 + 2], sample);
        }
        Ok(needed_bytes)
    }

    fn codec_type(&self) -> CodecType {
        CodecType::Pcm
    }
}

/// Linear PCM Decoder (identity uncompressed transport)
pub struct PcmAudioDecoder {
    profile: AudioProfile,
}

impl PcmAudioDecoder {
    pub fn new(profile: &AudioProfile) -> Self {
        Self {
            profile: profile.clone(),
        }
    }
}

impl AudioDecoder for PcmAudioDecoder {
    fn decode(&mut self, packet: Option<&[u8]>, output: &mut [i16], _fec: bool) -> Result<usize, CodecError> {
        let expected_samples = self.profile.total_samples_per_frame();
        if let Some(data) = packet {
            let available_samples = data.len() / 2;
            let samples_to_copy = available_samples.min(output.len()).min(expected_samples);
            for i in 0..samples_to_copy {
                output[i] = LittleEndian::read_i16(&data[i * 2..i * 2 + 2]);
            }
            Ok(samples_to_copy / self.profile.channels as usize)
        } else {
            // Silence concealment on packet loss
            output.fill(0);
            Ok(self.profile.samples_per_frame())
        }
    }

    fn codec_type(&self) -> CodecType {
        CodecType::Pcm
    }
}

/// Factory function to create matching encoder
pub fn create_encoder(profile: &AudioProfile) -> Result<Box<dyn AudioEncoder>, CodecError> {
    match profile.codec {
        CodecType::Opus => Ok(Box::new(OpusAudioEncoder::new(profile)?)),
        CodecType::Pcm => Ok(Box::new(PcmAudioEncoder::new(profile))),
    }
}

/// Factory function to create matching decoder
pub fn create_decoder(profile: &AudioProfile) -> Result<Box<dyn AudioDecoder>, CodecError> {
    match profile.codec {
        CodecType::Opus => Ok(Box::new(OpusAudioDecoder::new(profile)?)),
        CodecType::Pcm => Ok(Box::new(PcmAudioDecoder::new(profile))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn generate_sine_wave(sample_rate: u32, channels: u16, freq: f32, duration_ms: u32) -> Vec<i16> {
        let sample_count = ((sample_rate as f32 * (duration_ms as f32 / 1000.0)) as usize) * channels as usize;
        let mut samples = Vec::with_capacity(sample_count);
        let samples_per_channel = sample_count / channels as usize;

        for i in 0..samples_per_channel {
            let t = i as f32 / sample_rate as f32;
            let value = (2.0 * PI * freq * t).sin();
            let sample_i16 = (value * 16000.0) as i16;
            for _ in 0..channels {
                samples.push(sample_i16);
            }
        }
        samples
    }

    #[test]
    fn test_opus_encode_decode_roundtrip() {
        let profile = AudioProfile::default(); // 48kHz, 2ch, 20ms = 960 frames * 2 = 1920 samples
        let pcm_input = generate_sine_wave(profile.sample_rate, profile.channels, 440.0, profile.frame_duration_ms);
        assert_eq!(pcm_input.len(), 1920);

        let mut encoder = OpusAudioEncoder::new(&profile).expect("Failed to create Opus encoder");
        let mut decoder = OpusAudioDecoder::new(&profile).expect("Failed to create Opus decoder");

        let mut encoded_buf = vec![0u8; 1024];
        let bytes_written = encoder.encode(&pcm_input, &mut encoded_buf).expect("Encode failed");
        assert!(bytes_written > 10 && bytes_written <= 1000, "Encoded bytes: {}", bytes_written);

        let mut decoded_buf = vec![0i16; 1920];
        let decoded_frames = decoder.decode(Some(&encoded_buf[..bytes_written]), &mut decoded_buf, false)
            .expect("Decode failed");
        assert_eq!(decoded_frames, 960);
    }

    #[test]
    fn test_pcm_encode_decode_roundtrip() {
        let mut profile = AudioProfile::default();
        profile.codec = CodecType::Pcm;
        let pcm_input = generate_sine_wave(profile.sample_rate, profile.channels, 440.0, profile.frame_duration_ms);

        let mut encoder = PcmAudioEncoder::new(&profile);
        let mut decoder = PcmAudioDecoder::new(&profile);

        let mut encoded_buf = vec![0u8; 4000];
        let bytes_written = encoder.encode(&pcm_input, &mut encoded_buf).expect("PCM encode failed");
        assert_eq!(bytes_written, 1920 * 2);

        let mut decoded_buf = vec![0i16; 1920];
        decoder.decode(Some(&encoded_buf[..bytes_written]), &mut decoded_buf, false).expect("PCM decode failed");
        assert_eq!(pcm_input, decoded_buf);
    }

    #[test]
    fn test_opus_packet_loss_concealment() {
        let profile = AudioProfile::default();
        let mut decoder = OpusAudioDecoder::new(&profile).expect("Failed to create Opus decoder");
        let mut decoded_buf = vec![0i16; 1920];
        let plc_res = decoder.decode(None, &mut decoded_buf, false);
        assert!(plc_res.is_ok());
    }
}
