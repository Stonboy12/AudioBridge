# ADR-008: Smart Audio Routing dan Process Loopback Capture

**Status:** Diterima (Accepted)  
**Tanggal:** 6 November 2026  
**Penulis:** Senior Rust & Windows Audio Engineer  

---

## Konteks

Pada tahap sebelumnya (Stage 1A & Stage 2), AudioBridge hanya menangkap seluruh audio output desktop (System Audio Loopback).
Namun, PRD Stage 3 mensyaratkan:
1. Kemampuan memilih sumber audio secara independen per perangkat Android (misal Perangkat 1 mendengarkan Spotify, sementara Perangkat 2 mendengarkan audio sistem secara keseluruhan).
2. Deteksi kemampuan OS yang akurat: tidak menampilkan fitur per-aplikasi jika OS tidak mendukung.
3. Penanganan gracefully ketika proses yang sedang ditangkap ditutup oleh pengguna.

---

## Keputusan

1. **Dua Modus Capture WASAPI:**
   - **System Audio Loopback:** Menggunakan `IAudioClient` standard dengan `AUDCLNT_STREAMFLAGS_LOOPBACK` pada endpoint default audio renderer. Kompatibel dengan seluruh versi Windows (Windows 7/10/11).
   - **Process Loopback (Per-Application):** Menggunakan Windows 10 Build 20348+ / Windows 11 API `ActivateAudioInterfaceAsync` dengan parameter `AUDIOCLIENT_ACTIVATION_PARAMS` bertipe `AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK` dan target Process ID (PID).
2. **Capability Check Saat Startup:**
   - Deteksi versi build kernel OS saat inisialisasi aplikasi.
   - Bila build < 20348, Process Loopback dinonaktifkan di level engine dan di antarmuka pengguna diberi penanda ketidaktersediaan secara transparan, tanpa menyebabkan crash atau unhandled error.
3. **Mekanisme Fallback Otomatis:**
   - Ketika proses aplikasi yang sedang di-capture berhenti beroperasi (exit / crash / terminated), session manager mengalihkan perangkat yang mendengarkan proses tersebut kembali ke `src_sys` (System Audio) secara otomatis.
   - Mengirim notifikasi kontrol `SourceChanged` dengan flag `fallback: true` ke perangkat Android yang bersangkutan.

---

## Konsekuensi

### Positif:
- Setiap perangkat Android dapat mendengarkan audio yang terisolasi dari aplikasi yang diinginkan tanpa tercampur suara notifikasi OS atau audio aplikasi lain.
- Transisi fallback zero-downtime: stream audio tidak crash saat aplikasi desktop ditutup.
- Kepatuhan mutlak terhadap arsitektur Windows Audio tanpa ketergantungan driver virtual audio pihak ketiga yang tidak ditandatangani (seperti VB-Cable atau Virtual Audio Cable).

### Negatif / Keterbatasan:
- Capture per-aplikasi hanya tersedia di Windows 10 build 20348 (21H2 Server / Enterprise) atau Windows 11. Pengguna Windows 10 versi lama tetap dapat menggunakan System Audio loopback.
