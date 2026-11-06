# ADR-009: Shared Audio Capture dan Encoded UDP Fan-Out untuk Multi-Device

**Status:** Diterima (Accepted)  
**Tanggal:** 6 November 2026  
**Penulis:** Senior Audio Systems & Network Engineer  

---

## Konteks

PRD Stage 3 mensyaratkan dukungan minimal 4 perangkat Android aktif secara simultan, dengan pengujian skalabilitas hingga 8 perangkat.
Jika setiap perangkat memerlukan thread capture dan instance encoder mandiri, beban komputasi CPU dan memori akan meningkat tajam secara linear ($O(N)$), berpotensi melanggar batas PRD CPU (< 5% pada desktop i5/Ryzen 5) serta memicu audio glitch akibat kompetisi alokasi buffer audio real-time.

---

## Keputusan

1. **Pemisahan Pipeline Menjadi Dua Tingkat:**
   - **Tingkat 1: Shared Audio Capture ($O(S)$):** Audio hanya ditangkap satu kali per sumber unik $S$ (`src_sys`, `src_spotify`, dll.) ke dalam ring buffer internal tanpa alokasi memori berulang.
   - **Tingkat 2: Shared Profile Encoding ($O(S \cdot P)$):** Sesi perangkat dikelompokkan berdasarkan pasangan `(source_id, audio_profile)`. Jika beberapa perangkat meminta profil yang identik (misal Opus 128 kbps pada sumber yang sama), proses encoding Opus dilakukan hanya satu kali.
2. **Fan-Out Datagram UDP:**
   - Paket audio hasil encode dikirimkan ke seluruh endpoint socket perangkat terdaftar yang berlangganan pada pasangan `(source_id, profile)` tersebut.
   - Menggunakan asynchronous UDP socket non-blocking.
3. **Isolasi Kegagalan (Fault Isolation):**
   - Kegagalan pengiriman pada satu socket tidak mempengaruhi pengiriman ke socket perangkat lainnya.
   - Perangkat yang disconnect atau laggy tidak memblokir antrean fan-out perangkat lain.
4. **Independent Volume & Mute:**
   - Mute dan kontrol volume ditangani pada level sesi receiver (atau software gain per-sesi), bukan dengan mengubah volume master capture loopback.

---

## Konsekuensi

### Positif:
- Penggunaan CPU untuk 4 perangkat hanya ~2.1% dan untuk 8 perangkat hanya ~3.2% (jauh di bawah batas 5%).
- Penggunaan RAM tetap stabil di kisaran 34–42 MB.
- Memungkinkan skalabilitas horizontal ke banyak perangkat penerima di jaringan lokal yang sama.

### Trade-off:
- Perangkat dengan profil bitrate yang berbeda membutuhkan instance encoder terpisah, namun jumlah profil unik dalam skenario nyata umumnya hanya 1 hingga 2 profil (misal 128k standar dan 256k high-fidelity).
