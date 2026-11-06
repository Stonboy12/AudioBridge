# AudioBridge — Stage 0 Technical Validation Test Plan

**Tujuan:** Memvalidasi kelayakan teknis transmisi audio real-time dari PC Desktop ke Android melalui jaringan lokal (LAN) sebelum memasuki pengembangan MVP (Stage 1).

---

## 1. Matriks Rangkaian Pengujian (Test Suite)

| ID | Eksperimen | Metode / Deskripsi | Kriteria Lulus |
|---|---|---|---|
| **EXP-1** | Audio Capture Frame Assembly | Menangkap audio PCM 48 kHz stereo 16-bit (960 sample/channel per 20ms frame) dari audio engine | Data frame valid, timestamp berurutan, sample rate 48000 Hz, stereo |
| **EXP-2** | Opus Codec Roundtrip & PLC | Encode PCM ke Opus byte stream, lalu decode kembali; uji Packet Loss Concealment ketika paket None | Rasio kompresi > 8:1, decoding tanpa error, PLC menghasilkan audio sintetis halus |
| **EXP-3** | LAN Datagram Transport | Kirim audio datagram melalui UDP socket dengan header biner 29-byte | Header dan payload diterima utuh, sequence number dan timestamp terverifikasi |
| **EXP-4** | Jitter Buffer & Late Drop | Simulasikan paket out-of-order, packet loss 33%, dan paket terlambat | Jitter buffer menyusun ulang paket, mengabaikan paket usang, dan memicu sinyal conceal |
| **EXP-5** | Reconnection Backoff | Uji pemutusan koneksi soket dan pemulihan dengan exponential backoff | Interval backoff meningkat (500ms -> 1000ms -> 2000ms -> maks 8000ms) |
| **EXP-6** | Latency & Bitrate Profiling | Ukur durasi encode di CPU dan estimasi total end-to-end latency | Waktu encode < 2 ms, estimasi E2E latency di LAN terkontrol 35–65 ms |
| **EXP-7** | Continuous Streaming (10 Menit) | Alirkan audio secara kontinu selama minimal 600 detik (30.000 frame) dengan simulasi packet loss Wi-Fi 1.5% | Sesi streaming tidak mengalami crash, audio underrun < 0.1%, buffer stabil |
| **EXP-8** | Dependency License Audit | Tinjau seluruh dependensi Rust, C++, dan Android | 100% lisensi permisif (MIT, Apache-2.0, BSD-3-Clause) |

---

## 2. Cara Menjalankan Validasi

### A. Melalui Rust CLI Test Harness:
```bash
# 1. Menjalankan seluruh pengujian unit dan eksperimen cepat
cargo test

# 2. Menjalankan pengujian Stage 0 otomatis
cargo run --bin audiobridge-cli -- validate

# 3. Menjalankan 10-Minute Continuous Streaming Stress Test
cargo run --bin audiobridge-cli -- stress10
```

### B. Melalui Aplikasi Android:
1. Buka aplikasi **AudioBridge** di Android.
2. Navigasikan ke tab **Stage 0 Test Suite**.
3. Ketuk **"Run Full Technical Validation"** untuk menjalankan pengujian end-to-end langsung pada perangkat.
4. Ketuk **"10-Minute Continuous Streaming Stress Test"** untuk menguji ketahanan background service, konsumsi memori, dan underrun speaker.
