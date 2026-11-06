# ADR-002: Arsitektur Audio Output Android — Integrasi Oboe Native C++ dan Android AudioTrack

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Software Engineer (Android & Native Audio)  

## Konteks dan Masalah

Untuk Android audio receiver, latensi output audio dari saat paket didekode hingga speaker menghasilkan suara (*audio output pipeline latency*) merupakan komponen krusial dari target 50-100 ms end-to-end.

Android menyediakan dua jalur utama untuk audio output:
1. **Oboe C++ Library (Google):** Memanfaatkan AAudio (pada Android 8.1+ / API 27+) dengan fallback ke OpenSL ES pada versi lama. Menyediakan jalur *exclusive mode* MMAP low-latency dengan latensi serendah 10-15 ms.
2. **Android AudioTrack (Java/Kotlin API):** Standard platform API dengan `AudioAttributes.USAGE_MEDIA` dan `AudioTrack.PERFORMANCE_MODE_LOW_LATENCY` (`FLAG_LOW_LATENCY`).

## Evaluasi Teknis

### Oboe C++ Native Layer:
- **Keunggulan:** Latensi hardware terendah berkat akses langsung ke kernel MMAP buffer di perangkat yang mendukung AAudio.
- **Tantangan:** Memerlukan Android NDK, CMake, compiler toolchain C++17, dan binding JNI/FFI yang harus dikompilasi untuk 4 arsitektur ABI (`arm64-v8a`, `armeabi-v7a`, `x86`, `x86_64`). Di lingkungan container testing tertentu atau emulator tanpa NDK terinstal, kompilasi NDK dapat menjadi blocker.

### Android Low-Latency AudioTrack:
- **Keunggulan:** 100% kompatibel di semua versi Android, perangkat fisik, emulatur headless/browser, dan unit test Robolectric tanpa memerlukan toolchain NDK.
- **Performa:** Pada modern Android (API 28+), `AudioTrack` dengan mode low-latency menggunakan buffer underrun notification internal dengan latensi ~20-30 ms, yang sepenuhnya memenuhi kriteria Technical Validation Stage 0.

## Keputusan

Mengadopsi pola **Unified Audio Sink Architecture**:
1. Menghadirkan interface abstraksi `AudioSink` di Android (`AudioPlayerSink`).
2. Implementasi default di Android menggunakan **Low-Latency Streaming AudioTrack** dengan buffer adaptive underrun tracker. Ini memastikan aplikasi dapat langsung dibuild, dijalankan, dan dites di semua lingkungan (termasuk Android Streaming Emulator).
3. Menyediakan kode native C++ Oboe binding (`platform/android/native/OboeAudioSink.cpp`) dan spesifikasi CMake di repository, yang disiapkan untuk diaktifkan pada release build ber-NDK saat optimasi sub-20ms di Stage 1/Stage 2.

## Konsekuensi

- Memungkinkan eksekusi Technical Validation Stage 0 secara langsung tanpa hambatan toolchain NDK di lingkungan build container.
- Menjamin stabilitas dan reproduksibilitas pengujian audio di berbagai perangkat Android.
