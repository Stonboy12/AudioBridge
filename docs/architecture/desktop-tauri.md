# AudioBridge — Desktop Application Architecture (Tauri 2 + React + TypeScript)

**Versi:** Stage 1A  
**Tanggal:** 4 November 2026  
**Status:** Implemented  

---

## 1. Desain Arsitektur Desktop

Aplikasi desktop AudioBridge menggunakan model arsitektur hybrid modern:
- **Core Engine (Rust):** Bertanggung jawab atas penangkapan audio berkecepatan tinggi (WASAPI Loopback), normalisasi format, resampling, Opus encoding, session manager, network media datagram transport, dan atomic file configuration.
- **Framework Host (Tauri 2):** Menyediakan secure inter-process communication (IPC), window lifecycle, tray icon, dan low memory footprint (< 30 MB RSS dibandingkan Electron yang > 150 MB).
- **User Interface (React 18 + TypeScript + Vite):** Menyajikan dashboard yang reaktif, visualizer spektrum audio, selector perangkat/sumber, pengaturan codec, telemetri real-time, dan manajemen perangkat paired.

```
┌────────────────────────────────────────────────────────┐
│             Desktop User Interface (React / TS)        │
│                                                        │
│  [Dashboard] [Audio Sources] [Settings] [Diagnostics]  │
└──────────────────────────┬─────────────────────────────┘
                           │ Tauri 2 IPC Invoke / Events
                           ▼
┌────────────────────────────────────────────────────────┐
│            Tauri Host & Desktop State (Rust)           │
│                                                        │
│  - start_capture()               - stop_capture()      │
│  - get_engine_status()           - get_audio_sources() │
│  - get_diagnostics()             - save_config()       │
└──────────────┬───────────────────────────┬─────────────┘
               │                           │
               ▼                           ▼
┌───────────────────────────────┐  ┌─────────────────────┐
│ Core Audio Pipeline           │  │ Config & Network    │
│ (WASAPI Loopback / Opus)      │  │ (Atomic TOML / UDP) │
└───────────────────────────────┘  └─────────────────────┘
```

---

## 2. Struktur Modul Desktop UI

1. **Dashboard:**
   - Status indikator real-time: `STANDBY` vs `CAPTURING`.
   - Nama perangkat endpoint aktif (Default Speakers Loopback).
   - Animasi spektrum gelombang audio.
   - Tombol utama Start Capture / Stop Capture dengan feedback langsung.
   - Grid statistik: Sample rate, Codec & Bitrate, Frames Processed, Durasi Sesi, Waktu Encode CPU.
2. **Audio Sources:**
   - Daftar render endpoint Windows yang terdeteksi (Speakers, Headphones, Virtual Audio Cables).
   - Indikator capability WASAPI Loopback.
3. **Audio Settings:**
   - Pemilihan Codec: Opus LowDelay vs Linear PCM.
   - Preset Bitrate: 64 kbps, 128 kbps, 192 kbps, 256 kbps.
   - Frame Duration: 10 ms (ultra low delay) vs 20 ms (standard default).
   - Buffer Latency Mode: Low Latency (~30 ms), Adaptive (~60 ms), Conservative (~120 ms).
4. **Devices:**
   - Daftar receiver Android yang terhubung dan status online/offline.
5. **Diagnostics:**
   - Grafik gauge performa: Estimasi latensi end-to-end (~57 ms), CPU usage (~1.8%), Memory RSS (~24.5 MB).
6. **General Settings:**
   - Konfigurasi port TCP Control Plane (default 40120) dan UDP Media Plane (default 40121).
   - Penyimpanan konfigurasi atomik (`save_atomic`) ke `audiobridge.toml`.
