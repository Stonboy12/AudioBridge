# AudioBridge — Core Audio Engine Architecture (Stage 1A)

**Tanggal:** 4 November 2026  
**Status:** Implemented & Validated  
**Platform Target:** Windows Desktop (WASAPI Loopback) & Cross-Platform Core  

---

## 1. Arsitektur Pipeline Audio Desktop

Pipeline audio desktop AudioBridge didesain untuk menangkap sinyal audio sistem Windows secara real-time dengan latensi terendah yang aman (*bounded & predictable latency*), tanpa alokasi memori berulang di dalam callback audio (*zero-allocation callback path*):

```
┌─────────────────────────────────┐
│ Windows WASAPI Loopback Client  │
│ (IAudioCaptureClient / Loopback)│
└───────────────┬─────────────────┘
                │ Float32 Samples (Native WASAPI Mix Format)
                ▼
┌─────────────────────────────────┐
│ Audio Processing Engine         │
│  1. Float32 -> PCM16 Converter  │
│  2. Sinc / Linear Resampler     │ (e.g. 44.1/96 kHz -> 48 kHz canonical)
│  3. Channel Mapper (Mono/Stereo)│
│  4. Frame Assembler (20ms/960s) │
└───────────────┬─────────────────┘
                │ Canonical Interleaved PCM16 Frames
                ▼
┌─────────────────────────────────┐
│ Opus Encoder Wrapper            │
│ (Application::LowDelay 128kbps) │
└───────────────┬─────────────────┘
                │ Encoded Audio Packets (avg ~320 bytes)
                ▼
┌─────────────────────────────────┐
│ Bounded FIFO Queue (Backpressure)│
│ (Capacity: 50 frames = 1.0 sec) │ (Drops oldest frame if network stalls)
└───────────────┬─────────────────┘
                │ 30-Byte Datagram Header + Payload
                ▼
┌─────────────────────────────────┐
│ Media Datagram Transport (UDP)  │
│ (Socket Pacing -> Android LAN)  │
└─────────────────────────────────┘
```

---

## 2. Modul-Modul Inti

### A. `crates/audio-capture`
- Mengabstraksikan antarmuka `AudioCaptureEngine`.
- Pada Windows: Menggunakan COM API `IMMDeviceEnumerator` untuk memperoleh default console render endpoint (`eRender`, `eConsole`), mengaktifkan `IAudioClient` dalam mode `AUDCLNT_STREAMFLAGS_LOOPBACK`, dan membaca frame audio melalui `IAudioCaptureClient`.
- Menangani flag `AUDCLNT_BUFFERFLAGS_SILENT` dengan mengisi buffer ke nol (*silence padding*) agar tidak mentransmisikan noise/sampah memori.
- Menangani `AUDCLNT_E_DEVICE_INVALIDATED` dengan pemulihan re-enumerasi otomatis.
- Menyediakan `SyntheticLoopbackCapture` beresolusi tinggi untuk pengujian deterministik dan cross-platform.

### B. `crates/audio-processing`
- `float32_to_pcm16`: Konversi floating-point IEEE 32-bit (range `[-1.0, 1.0]`) ke 16-bit signed integer dengan saturasi aman (`[-32768, 32767]`).
- `Resampler`: Mengonversi sample rate perangkat audio (misal 44.100 Hz, 96.000 Hz, 192.000 Hz) ke format kanonikal 48.000 Hz dengan interpolasi fraksional anti-aliasing.
- `FrameAssembler`: Mengumpulkan sampel audio ke ukuran frame diskret yang konsisten (20 ms = 960 sampel/channel stereo = 1.920 sampel total per paket).
- `AudioRingBuffer`: FIFO ring buffer bounded untuk transfer data antar thread.

### C. `crates/audio-codec`
- Mengabstraksikan trait `AudioEncoder` dan `AudioDecoder`.
- Mengimplementasikan `OpusAudioEncoder` berbasis libopus reference dengan mode `Application::LowDelay`.
- Mengaktifkan *in-band Forward Error Correction (FEC)* dan *Packet Loss Concealment (PLC)*.
- Menyediakan fallback `PcmAudioEncoder` untuk kebutuhan verifikasi audio mentah dan benchmarking.

### D. `crates/audio-pipeline`
- Mengelola orkestrasi siklus hidup: start, stop, dan monitor.
- Mengimplementasikan mekanisme backpressure: jika queue mencapai batas maksimum (50 frame / 1 detik), frame tertua dibuang untuk menjaga latensi tetap real-time tanpa penundaan akumulatif.
- Mengumpulkan metrik performa: durasi encoding per frame dalam mikrodetik, jumlah frame diproses, frame dibuang, dan durasi sesi aktif.
