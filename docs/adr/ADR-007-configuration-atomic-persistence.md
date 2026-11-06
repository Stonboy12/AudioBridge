# ADR-007: Mekanisme Penyimpanan Konfigurasi Atomik Tanpa Database

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Software Engineer (Core Architecture)  

## Konteks

Sesuai PRD Seksi 15, MVP AudioBridge dirancang tanpa database (tanpa SQLite atau relational DB) karena AudioBridge adalah aplikasi utilitas streaming, bukan aplikasi transaksi data besar.

Namun, penyimpanan file konfigurasi biasa (`fs::write`) rentan terhadap korupsi data jika terjadi aplikasi crash, sistem shutdown mendadak, atau pemutusan daya di tengah-tengah operasi I/O penulisan berkas.

## Keputusan

1. **Format File Konfigurasi:** Menggunakan **TOML** (`audiobridge.toml`) dengan field wajib `schema_version = 1`.
2. **Skema dan Validasi:** Setiap pembacaan atau penyimpanan konfigurasi divalidasi terhadap batasan nilai yang diizinkan (port unprivileged 1024–65535, bitrate 32–512 kbps, sample rate 44.1k/48k/96k, frame duration 10/20 ms).
3. **Pola Penulisan Atomik (`save_atomic`):**
   - File konfigurasi terlebih dahulu ditulis ke file temporer anonim (`tempfile::NamedTempFile`) di direktori yang sama.
   - Buffer diflush secara sinkron ke disk (`temp_file.flush()`).
   - File temporer dipersist/direname secara atomik menggunakan instruksi rename filesystem kernel (`std::fs::rename`).
   - Sifat operasi rename filesystem atomik di sistem POSIX maupun Windows menjamin berkas konfigurasi tidak akan pernah berada dalam kondisi setengah tertulis atau korup.
4. **Pemisahan Secret:** Kunci privat dan token tidak disimpan dalam file konfigurasi plain TOML, melainkan menggunakan OS Credential Store (Windows Credential Manager / Android Keystore).

## Konsekuensi

- Mencegah rusaknya konfigurasi pengguna saat PC mati mendadak.
- Menghindari ketergantungan library database eksternal yang memperberat binary aplikasi.
