# Arsitektur Audio Profile Management AudioBridge

**Status:** Produksi (Stage 3)  
**Komponen:** Desktop Engine (`audio-types`, `audio-codec`, `session`), Android Receiver (`AudioReceiverModel`, `ReceiverViewModel`)

---

## 1. Tujuan & Ruang Lingkup

Audio Profile Management memungkinkan setiap perangkat Android menegosiasikan dan menyimpan parameter streaming yang paling sesuai dengan kondisi jaringan dan preferensi pendengarnya:
- **Codec**: Opus (Interactive Low-Delay) atau Linear PCM (Uncompressed Diagnostic).
- **Bitrate**: 64, 96, 128, 192, atau 256 kbps.
- **Buffer Mode**: Low Latency (~30 ms), Adaptive (~60 ms), atau Conservative (~120 ms).
- **Audio Routing State**: Volume level (0.0 – 1.0) dan Status Mute (`is_muted: bool`).

---

## 2. Struktur Profil Audio

### Definisi Android (Kotlin)
```kotlin
data class AudioProfileConfig(
    val codec: AudioCodec = AudioCodec.OPUS,
    val sampleRate: Int = 48000,
    val channels: Int = 2,
    val bitrateKbps: Int = 128,
    val frameDurationMs: Int = 20,
    val bufferMode: BufferMode = BufferMode.ADAPTIVE,
    val isMuted: Boolean = false
) {
    val samplesPerFrame: Int get() = (sampleRate * frameDurationMs) / 1000
    val totalSamplesPerFrame: Int get() = samplesPerFrame * channels
}
```

### Definisi Desktop Engine (Rust)
```rust
pub struct AudioProfile {
    pub codec: CodecType,         // Opus | Pcm
    pub sample_rate: u32,         // 48000
    pub channels: u16,            // 2
    pub bitrate_kbps: u32,        // 64..256
    pub frame_duration_ms: u32,   // 20
    pub buffer_mode: BufferMode,  // LowLatency | Adaptive | Conservative
}
```

---

## 3. Presets Buffer & Latency Tuning

| Mode Buffer | Target Delay | Karakteristik | Kasus Penggunaan Ideal |
|---|---|---|---|
| **Low Latency** | ~30 ms | Buffer tipis (1–2 frame), drop paket terlambat agresif | Gaming, sinkronisasi video/film |
| **Adaptive** | ~60 ms | Buffer dinamis berbasis estimasi varians jitter real-time | Musik harian, streaming normal di Wi-Fi rumah |
| **Conservative** | ~120 ms | Buffer dalam (5–6 frame), PLC prioritas tinggi | Jaringan Wi-Fi padat/ramai atau jangkauan jauh |

---

## 4. Persistensi Konfigurasi

Sesuai ADR-007:
1. **Desktop**: Konfigurasi per-perangkat disimpan secara atomik pada file TOML lokal (`audiobridge.toml`) tanpa menggunakan database eksternal.
2. **Android**: Preferensi profil (bitrate, buffer mode, mute) disimpan menggunakan Android DataStore, sehingga preferensi tetap terjaga setelah aplikasi ditutup dan dibuka kembali.

---

## 5. Sinkronisasi Kontrol Pesawat

Ketika pengguna mengubah bitrate, buffer mode, atau volume/mute di Android:
1. State lokal `ReceiverUiState` langsung diperbarui secara responsif.
2. Android mengirim pesan `SetProfile` atau `SetDeviceMute` ke Desktop melalui TCP Control Channel:
   ```json
   {
     "type": "SetProfile",
     "payload": {
       "codec": "Opus",
       "bitrate_kbps": 192,
       "buffer_mode": "Adaptive"
     }
   }
   ```
3. Desktop memperbarui `SessionInfo` untuk client tersebut. Jika profil encoder baru belum ada di kumpulan fan-out, encoder baru diinisialisasi secara transparan.
