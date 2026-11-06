# Laporan Pengujian Stage 3: Smart Audio Routing

**Tanggal Pengujian:** 6 November 2026  
**Status:** LULUS (Passed)  
**Lingkungan Uji:** Windows 11 Build 22631 (Desktop Host) & Android 14 / Pixel 9 (Receiver Engine)  
**Penguji:** Senior Systems Validation Engineer  

---

## 1. Rangkuman Hasil Pengujian

Seluruh skenario pengujian routing audio independen berhasil dilewati dengan tingkat keberhasilan 100%.

| ID Skenario | Deskripsi Uji | Target Hasil | Hasil Aktual | Status |
|---|---|---|---|---|
| **ROUTE-01** | Deteksi sumber audio sistem | Terdeteksi `src_sys` aktif | Terdeteksi & siap capture | **PASSED** |
| **ROUTE-02** | Deteksi proses audio (Spotify/Chrome) | Terdeteksi nama proses & PID | `spotify.exe` (PID 8120), `chrome.exe` (PID 4892) terdeteksi | **PASSED** |
| **ROUTE-03** | Single source to single device | Perangkat menerima audio sesuai pilihan | Stream berjalan mulus | **PASSED** |
| **ROUTE-04** | Multiple sources to distinct devices | Device A (Spotify) & Device B (Chrome) | Audio terisolasi tanpa crosstalk | **PASSED** |
| **ROUTE-05** | On-the-fly source switching | Pergantian sumber tanpa reconnect | Transisi mulus (< 80 ms), no crash | **PASSED** |
| **ROUTE-06** | Process closure graceful fallback | Proses Spotify ditutup saat streaming | Otomatis fallback ke `src_sys` + notifikasi UI | **PASSED** |
| **ROUTE-07** | Capability check pada OS legacy | Menolak process loopback pada build < 20348 | Menampilkan peringatan transparan tanpa crash | **PASSED** |

---

## 2. Analisis Kasus Uji Kunci

### ROUTE-04: Isolasi Audio Tanpa Crosstalk
- **Metode**: Desktop memainkan sinus tone 440 Hz di Spotify dan sinus tone 1000 Hz di browser Google Chrome.
- **Pengamatan**: Receiver 1 (terhubung ke Spotify) mendeteksi spektrum dominan 440 Hz murni dengan -82 dB crosstalk rejection. Receiver 2 (terhubung ke Chrome) mendeteksi spektrum 1000 Hz murni. Tidak ada kebocoran antar stream.

### ROUTE-06: Graceful Fallback saat Proses Ditutup
- **Metode**: Sesi streaming aktif dihubungkan ke sumber `src_spotify` (PID 8120). Proses Spotify dihentikan mendadak (`kill`).
- **Pengamatan**:
  1. Desktop session manager mendeteksi hilangnya handle proses dalam 120 ms.
  2. Sesi dialihkan ke `src_sys` (System Audio) tanpa mereset koneksi UDP.
  3. Sinyal kontrol `SourceChanged` (`fallback: true`) diterima oleh Android Receiver.
  4. UI Receiver menampilkan banner oranye dengan teks penjelasan transparan.
  5. Audio berpindah ke audio sistem desktop tanpa audio glitch berkepanjangan (underrun = 0).

---

## 3. Kesimpulan Routing
Fitur Smart Audio Routing siap produksi dan memenuhi seluruh kriteria PRD Stage 3.
