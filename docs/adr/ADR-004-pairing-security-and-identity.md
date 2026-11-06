# ADR-004: Mekanisme Pairing, Identitas Perangkat, dan Keamanan

**Status:** Diterima (Accepted)  
**Tanggal:** 4 November 2026  
**Penulis:** Senior Software Engineer (Security & Networking)  

## Konteks

AudioBridge beroperasi di jaringan lokal (LAN) seperti Wi-Fi rumah atau kantor. Pengguna lain di jaringan yang sama tidak boleh mendengarkan audio sistem PC secara diam-diam tanpa otorisasi. Di sisi lain, proses pairing harus cepat, intuitif, dan tidak membutuhkan akun cloud eksternal.

## Keputusan

1. **One-Time PIN Code:**
   - Ketika perangkat Android pertama kali terhubung ke Desktop, desktop menghasilkan kode numerik 6 digit acak yang ditampilkan di layar desktop (misal: `849 201`).
   - Kode berlaku selama 3 menit dan memiliki batas percobaan maksimum 3 kali gagal untuk mencegah brute force attack.
   - Desktop menampilkan dialog persetujuan (*Approval Dialog*) dengan nama perangkat Android.
2. **Device Identity Key (Asymmetric Key Pair):**
   - Setelah persetujuan, perangkat Android dan Desktop bertukar identitas publik (ed25519 public key atau SHA-256 fingerprint).
   - Di Android, private key disimpan di **Android Keystore**, sedangkan metadata desktop tepercaya disimpan di Jetpack DataStore.
   - Sesi streaming berikutnya secara otomatis terautentikasi tanpa harus memasukkan kode PIN ulang.
3. **Revocation:**
   - Pengguna desktop dapat mencabut izin (*Revoke*) perangkat kapan saja dari antarmuka "Trusted Devices", yang secara langsung membatalkan token autentikasi.
