# ADR-005: Strategi Audit Lisensi Dependensi untuk Distribusi Proprietary

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Software Engineer (Open Source Compliance & Architecture)  

## Konteks

AudioBridge adalah produk komersial closed source. Penggunaan pustaka open source pihak ketiga harus mematuhi batasan hukum hak cipta untuk mencegah risiko kewajiban lisensi copyleft (seperti GPLv2, GPLv3, AGPL) yang menuntut pembukaan kode sumber aplikasi.

## Keputusan

1. **Whitelisted License Classes:**
   - Lisensi permisif: `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Unlicense`, `Zlib`.
2. **Prohibited License Classes:**
   - Lisensi copyleft kuat: `GPL-*`, `AGPL-*`, `CPAL`, `OSL`, `SSPL`.
   - Kode sumber dari proyek GPL lain (seperti Gemacast atau sejenisnya) dilarang disalin atau diadaptasi secara langsung ke repository AudioBridge.
3. **Automated License Inventory:**
   - Menyimpan dokumen `docs/dependencies/license-inventory.md` yang mencakup seluruh dependensi Rust, Android, dan C++.
   - Setiap rilis binary (APK atau desktop installer) menyertakan berkas atribusi lisensi pihak ketiga sesuai persyaratan Apache-2.0 Seksi 4 dan BSD-3-Clause Klausul 2.
