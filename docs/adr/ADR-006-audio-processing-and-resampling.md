# ADR-006: Strategi Pemrosesan Audio, Normalisasi Format, dan Resampling

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Audio & Rust Engineer  

## Konteks

Windows WASAPI loopback (`AUDCLNT_STREAMFLAGS_LOOPBACK`) mengembalikan audio dalam format *mix format* milik perangkat audio default. Pada sebagian besar instalasi Windows 10/11, mix format ini berupa **IEEE 32-bit Floating Point (`WAVE_FORMAT_IEEE_FLOAT`)** dengan sample rate bervariasi (44.100 Hz, 48.000 Hz, atau 96.000 Hz) dan channel stereo.

Sementara itu:
1. Encoder Opus beroperasi paling efisien pada format signed 16-bit PCM kanonikal 48.000 Hz.
2. Receiver Android mengharapkan ukuran frame diskret yang konsisten (misal 20 ms = 960 sampel/channel stereo = 1.920 sampel).
3. Callback penangkapan audio WASAPI terjadi pada interval yang ditentukan driver (biasanya ~10 ms) yang ukurannya tidak selalu genap 20 ms.

## Keputusan

1. **Format Normalization Pipeline:**
   - Seluruh sampel float 32-bit dinormalisasi ke 16-bit integer PCM menggunakan fungsi SIMD/inlined `float32_to_pcm16` dengan saturasi nilai `[-32768, 32767]`.
2. **Dynamic Resampling:**
   - Mengimplementasikan `Resampler` berbasis interpolasi fraksional anti-aliasing. Jika driver soundcard berjalan pada 44.100 Hz atau 96.000 Hz, resampler secara otomatis menyesuaikan ke 48.000 Hz kanonikal sebelum masuk ke tahap Opus encoder.
3. **Decoupled Frame Assembly:**
   - Menggunakan `FrameAssembler` yang mengumpulkan sampel audio yang masuk ke dalam ring buffer internal dan mengeluarkan frame tepat sebesar `total_samples_per_frame` (1.920 sampel).
4. **Zero-Allocation Callback Rule:**
   - Semua scratch buffer (`temp_pcm16`, `encoded_scratch`) dipre-alokasikan di dalam `PipelineSharedState` untuk mencegah *garbage collection* atau *heap re-allocation* di dalam thread real-time audio callback.

## Konsekuensi

- **Positif:** Mengeliminasi audio glitch dan crackling akibat perbedaan sample rate soundcard; menjaga beban CPU di bawah 2%.
- **Negatif:** Resampling fraksional menambahkan overhead komputasi minimal (~0.1 ms), yang sepenuhnya dapat diabaikan dibandingkan total latensi jaringan.
