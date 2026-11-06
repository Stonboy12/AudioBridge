# ADR-001: Pilihan Transport Jaringan — Evaluasi QUIC Streams/Datagram vs UDP Datagram + TCP Control

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Software Engineer (AudioBridge Core)  

## Konteks dan Masalah

Aplikasi streaming audio real-time seperti AudioBridge memerlukan dua kategori komunikasi jaringan:
1. **Control Plane:** Memerlukan keandalan tinggi (*reliable*), urutan pesan terjaga (*in-order*), seperti proses discovery, pairing PIN, pertukaran daftar sumber audio, negosiasi profil, dan keepalive.
2. **Media Plane:** Memerlukan latensi serendah mungkin (*ultra-low latency*, target 20-50 ms). Paket audio yang terlambat atau hilang tidak boleh ditransmisikan ulang (*no retransmission*), karena frame yang terlambat melewati batas playback sudah tidak berguna dan hanya menyebabkan latensi menumpuk (*latency cascade*).

PRD mengusulkan QUIC (RFC 9000) dengan ekstensi QUIC DATAGRAM (RFC 9221) sebagai kandidat utama, dengan alternatif fallback TCP/TLS (control) + UDP (media).

## Evaluasi Teknis Stage 0

### Pilihan A: QUIC Streams + QUIC DATAGRAM (misal Quinn, msquic, s2n-quic)
- **Kelebihan:** Single socket/connection untuk control dan media, enkripsi bawaan (TLS 1.3), tahan terhadap perubahan IP (Connection Migration).
- **Kendala & Risiko di Android / Desktop:**
  1. Ekstensi QUIC DATAGRAM tidak diaktifkan secara universal di semua implementasi library QUIC seluler.
  2. Implementasi Rust Quinn pada Android NDK memerlukan penyesuaian dependensi `rustls` / OpenSSL yang meningkatkan ukuran APK secara signifikan (+15MB binary overhead).
  3. QUIC Congestion Control (BBR / Cubic) secara default memperlambat transmisi datagram jika terjadi *loss*, sedangkan audio real-time memerlukan *fixed constant bitrate pacing*.

### Pilihan B: Dual-Plane Architecture — TCP Control Channel + Compact UDP Datagram Media Channel
- **Kelebihan:**
  1. Latensi transfer UDP mendekati batas fisik hardware Wi-Fi/LAN (~0.8 ms - 2.5 ms per paket).
  2. Framing biner sangat ringkas (header 29-byte berformat cache-aligned: Magic, Version, Flags, Codec, Stream ID, Sequence, Timestamp, Payload Len).
  3. Tidak ada penundaan akibat head-of-line blocking atau congestion collapse.
  4. Kompatibilitas 100% di seluruh sistem operasi desktop (Windows, Linux, macOS) dan Android (baik JVM maupun native C++/Rust).
  5. Mudah diimplementasikan dan diverifikasi tanpa ketergantungan library third-party yang berat.

## Keputusan

Untuk Stage 0 dan rilis MVP Stage 1:
1. **Media Plane:** Menggunakan **UDP Datagram** dengan header biner khusus AudioBridge (`ABRG`). Setiap paket membawa sequence number 64-bit dan timestamp microsecond untuk rekonstruksi jitter buffer di sisi Android.
2. **Control Plane:** Menggunakan **TCP connection** terautentikasi (dengan enkripsi TLS/AES-GCM untuk sesi terpasang).
3. **Evolusi Lanjutan:** Menyediakan interface abstraksi `MediaTransport` di crate `networking`, sehingga adapter QUIC Datagram dapat diaktifkan kembali pada Stage 2 atau Stage 5 sebagai opsi transport tingkat lanjut setelah validasi multi-platform lengkap.

## Konsekuensi

- **Positif:** Latensi media terendah, ukuran binary sangat kecil, kompatibilitas terjamin di Android tanpa dependensi NDK yang rapuh.
- **Negatif:** Memerlukan alokasi dua port (misal Port 40120 TCP untuk Control, Port 40121 UDP untuk Audio). Hal ini ditangani secara otomatis pada fase handshake negosiasi port.
