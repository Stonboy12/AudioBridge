# ADR-003: Pemilihan Codec Audio — Opus vs Linear PCM

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Software Engineer (Audio Processing & Codec)  

## Konteks

Transmisi audio PCM mentah 48 kHz stereo 16-bit membutuhkan bandwidth jaringan:
`48000 samples/s * 2 channels * 2 bytes/sample * 8 bits = 1.536 Mbps`.
Meskipun bandwidth LAN modern mampu menampung 1.5 Mbps, lalu lintas paket kontinu dengan ukuran 3840 bytes per frame (frame 20 ms) sangat rentan terhadap fragmentasi IP, jitter Wi-Fi, dan penurunan performa saat beberapa perangkat terhubung simultan.

## Evaluasi Teknis

### Opus Audio Codec (RFC 6716):
- **Efisiensi Bandwidth:** Mampu mengompresi audio PCM 1536 kbps menjadi 64–192 kbps (rasio kompresi 8:1 hingga 24:1) dengan kualitas akustik transparansi tinggi (*imperceptible loss*).
- **Latensi Algoritmik:** Pada mode `Application::LowDelay` dan frame duration 20 ms, latensi encoding di CPU desktop modern terukur hanya 0.8–1.5 ms.
- **Resiliensi Jaringan:** Opus memiliki dukungan *in-band Forward Error Correction (FEC)* dan *Packet Loss Concealment (PLC)* bawaan. Ketika paket hilang di jaringan Wi-Fi, decoder dapat memprediksi bentuk gelombang audio yang hilang tanpa terdengar suara klik atau *drop* yang kasar.

### Linear PCM (Uncompressed):
- **Keunggulan:** Latensi encoding/decoding praktis nol (0 ms) tanpa distorsi kompresi.
- **Kekurangan:** Ukuran payload besar (3840 bytes per 20ms frame, melebihi MTU Ethernet standar 1500 bytes sehingga mengalami IP packet fragmentation).

## Keputusan

1. **Default Codec:** **Opus** ditetapkan sebagai codec utama default AudioBridge untuk semua profil koneksi normal, dengan frame duration 20 ms (960 samples per channel pada 48 kHz).
2. **Profil Bitrate:**
   - *Low Latency / Bandwidth Savings:* 64 kbps (mono/stereo dialog)
   - *Normal (Default):* 128 kbps (high quality music & system sound)
   - *High Fidelity:* 192 kbps (audiophile reference)
3. **Format Alternatif:** **Linear PCM (16-bit LE)** dipertahankan sebagai opsi diagnostik dan backward compatibility ketika debugging audio artifact.
