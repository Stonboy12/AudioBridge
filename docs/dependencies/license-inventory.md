# AudioBridge — License Inventory & Compatibility Audit

**Tanggal:** 5 November 2026  
**Status:** Stage 0, Stage 1A, Stage 1B, and Stage 2 Validated for Closed Source / Proprietary Distribution  
**Lisensi Produk AudioBridge:** Proprietary / Closed Source  

---

## 1. Ringkasan Kebijakan Lisensi

AudioBridge adalah produk perangkat lunak komersial/proprietary. Sesuai PRD Seksi 25, seluruh dependensi (Rust, Android/Kotlin, C/C++) harus diaudit secara ketat sebelum diadopsi:
- **DILARANG:** Dependensi berlisensi GPL, AGPL, CPAL, atau lisensi copyleft kuat lainnya yang mewajibkan pembukaan source code produk utama (*viral copyleft*).
- **DIIZINKAN DENGAN SYARAT:** Lisensi permisif (MIT, Apache 2.0, BSD-2-Clause, BSD-3-Clause, ISC, zlib) dengan menyertakan notice/attribution pada dokumen distribusi (About / Open Source Licenses screen).
- **LISENSI LEMAH (LGPL):** Dihindari sedapat mungkin. Jika digunakan dalam bentuk dynamic linking (shared library), pastikan tidak ada kode LGPL yang terkompilasi statis ke dalam binary aplikasi utama.

---

## 2. Inventaris Dependensi Rust (Shared Core, Networking & Desktop)

| Crate | Versi | Lisensi | Fungsi | Kompatibilitas Proprietary | Catatan Attribution |
|---|---|---|---|---|---|
| `opus` | 0.4.0 | MIT / Apache-2.0 | High-level Rust binding untuk libopus | ✅ Kompatibel | Sertakan file LICENSE |
| `opusic-sys` | 0.7.7 | BSD-3-Clause (libopus) | Native C wrapper untuk Reference Opus | ✅ Kompatibel | BSD-3-Clause permisif, aman untuk komersial |
| `byteorder` | 1.5.0 | MIT / Unlicense | Endian conversion untuk network framing | ✅ Kompatibel | Tanpa restriksi |
| `bytes` | 1.6.0 | MIT | Zero-copy byte buffer manipulation | ✅ Kompatibel | Permisif |
| `uuid` | 1.8.0 | MIT / Apache-2.0 | 128-bit session and challenge identifier | ✅ Kompatibel | Permisif |
| `ring` | 0.17.8 | ISC / MIT / OpenSSL | CSPRNG secure random token generation | ✅ Kompatibel | Permisif (OpenSSL/ISC) |
| `thiserror` | 1.0.60 | MIT / Apache-2.0 | Error handling derive macros | ✅ Kompatibel | Permisif |
| `serde` | 1.0.200 | MIT / Apache-2.0 | Serialization framework | ✅ Kompatibel | Permisif |
| `serde_json` | 1.0.120 | MIT / Apache-2.0 | JSON control plane protocol | ✅ Kompatibel | Permisif |
| `tokio` | 1.38.0 | MIT | Asynchronous runtime & networking | ✅ Kompatibel | Permisif |
| `parking_lot` | 0.12.3 | MIT / Apache-2.0 | High-performance sync primitives | ✅ Kompatibel | Permisif |
| `log` | 0.4.22 | MIT / Apache-2.0 | Structured logging facade | ✅ Kompatibel | Permisif |
| `env_logger` | 0.11.3 | MIT / Apache-2.0 | Console logger implementation | ✅ Kompatibel | Permisif |
| `rand` | 0.8.5 | MIT / Apache-2.0 | Random number generator (jitter simulation & pairing) | ✅ Kompatibel | Permisif |
| `chrono` | 0.4.38 | MIT / Apache-2.0 | Timestamping & diagnostics duration | ✅ Kompatibel | Permisif |
| `toml` | 0.8.19 | MIT / Apache-2.0 | Parsing & serialisasi TOML file konfigurasi | ✅ Kompatibel | Permisif |
| `tempfile` | 3.10.1 | MIT / Apache-2.0 | Berkas sementara untuk operasi penulisan atomik | ✅ Kompatibel | Permisif |

---

## 2.1 Inventaris Dependensi Desktop Frontend (React & TypeScript)

| Package | Versi | Lisensi | Fungsi | Kompatibilitas Proprietary |
|---|---|---|---|---|
| `react` | 18.3.1 | MIT | Komponen UI reaktif Desktop | ✅ Kompatibel |
| `react-dom` | 18.3.1 | MIT | DOM renderer untuk React UI | ✅ Kompatibel |
| `vite` | 5.4.10 | MIT | Build tool dan bundler UI desktop | ✅ Kompatibel |
| `typescript` | 5.6.3 | Apache-2.0 | Type checking dan kompilasi TypeScript | ✅ Kompatibel |

## 3. Inventaris Dependensi Android (Kotlin & Native)

| Library / SDK | Versi | Lisensi | Fungsi | Kompatibilitas Proprietary | Catatan Attribution |
|---|---|---|---|---|---|
| `androidx.compose.*` | BOM 2024.09.00 | Apache-2.0 | Jetpack Compose Declarative UI | ✅ Kompatibel | Standar Android, Apache 2.0 |
| `androidx.core:core-ktx` | 1.18.0 | Apache-2.0 | Android KTX extensions | ✅ Kompatibel | Apache 2.0 |
| `androidx.lifecycle:*` | 2.8.7 | Apache-2.0 | ViewModel & Lifecycle Coroutines | ✅ Kompatibel | Apache 2.0 |
| `androidx.datastore:*` | 1.1.7 | Apache-2.0 | Preferences persistence | ✅ Kompatibel | Apache 2.0 |
| `kotlinx-coroutines-*` | 1.10.2 | Apache-2.0 | Asynchronous Coroutines & Flow | ✅ Kompatibel | Apache 2.0 |
| `google.android.material` | M3 | Apache-2.0 | Material 3 Components & Icons | ✅ Kompatibel | Apache 2.0 |
| `android.net.nsd.NsdManager` | OS Native | Apache-2.0 / AOSP | Zero-conf mDNS discovery | ✅ Kompatibel | API resmi Android OS |
| `android.media.AudioTrack` | OS Native | Apache-2.0 / AOSP | Hardware low-latency PCM playback | ✅ Kompatibel | API resmi Android OS |
| `Oboe` (C++) | 1.9.0 | Apache-2.0 | High-performance C++ audio wrapper | ✅ Kompatibel | Apache 2.0 (Google AOSP) |

---

## 4. Evaluasi Libopus (Codec Utama)

Format codec Opus (RFC 6716) dikembangkan oleh IETF / Xiph.Org Foundation / Mozilla / Skype.
- **Lisensi Libopus:** BSD 3-Clause Revised.
- **Paten:** Bebas royalti (*royalty-free*) di bawah IETF RFC 6716.
- **Kesimpulan Legal:** Lisensi BSD-3-Clause sepenuhnya kompatibel dengan aplikasi proprietary/closed source, baik dilink secara statis maupun dinamis, asalkan teks hak cipta (*copyright notice*) disertakan dalam dokumentasi distribusi.

---

## 5. Keputusan Audit Lisensi

Seluruh dependensi yang dievaluasi dan digunakan pada Stage 0, Stage 1A, Stage 1B, dan Stage 2 memiliki lisensi **MIT**, **Apache-2.0**, atau **BSD-3-Clause**. Tidak ditemukan ketergantungan GPL atau lisensi berisiko lainnya. Strategi lisensi proprietary AudioBridge **Dinyatakan Valid dan Lulus**.
