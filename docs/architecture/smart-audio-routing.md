# Arsitektur Smart Audio Routing AudioBridge

**Status:** Produksi (Stage 3)  
**Komponen:** Desktop Engine (`audio-capture`, `audio-types`, `networking`), Android Receiver (`ReceiverViewModel`, `AudioBridgeApp`)

---

## 1. Ringkasan Eksekutif

Smart Audio Routing memungkinkan setiap perangkat Android penerima (Receiver) memilih dan mendengarkan sumber audio desktop secara independen. Sistem mendukung:
1. **System Audio Loopback**: Menangkap seluruh audio output Windows secara global melalui WASAPI loopback capture (`eRender`, `eMultimedia`).
2. **Process Loopback (Per-Application Audio)**: Menangkap audio spesifik dari satu aplikasi (misal Spotify, Chrome, Discord, game) menggunakan API Windows Audio Session API (WASAPI) Process Loopback (`AUDIOCLIENT_ACTIVATION_PARAMS` dengan `PROCESS_LOOPBACK_MODE`).

---

## 2. Deteksi Kemampuan OS (Capability Detection)

Windows menyediakan dua paradigma audio loopback:
- **Legacy WASAPI Loopback (Windows 7–10 build < 20348)**: Hanya mendukung full system output audio loopback via `IAudioClient::Initialize` dengan flag `AUDCLNT_STREAMFLAGS_LOOPBACK`.
- **Modern Process Loopback (Windows 10 Build 20348+ dan Windows 11)**: Mendukung capture loopback per-PID menggunakan `ActivateAudioInterfaceAsync` dengan `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK`.

### Aturan Integritas Antarmuka
Sesuai PRD Seksi 9 dan panduan Stage 3:
- AudioBridge **tidak pernah** menampilkan fitur yang tidak didukung oleh versi OS pengguna.
- Sistem mengecek build Windows saat runtime (`GetVersionEx` / `RtlGetVersion`). Jika build < 20348, UI secara eksplisit menandai mode Process Audio sebagai tidak tersedia dengan penjelasan transparan: *"Process Loopback requires Windows 10 Build 20348+ or Windows 11"*.

---

## 3. Model Sumber Audio (`AudioSourceInfo` / `AudioSourceItem`)

Setiap sumber audio direpresentasikan dengan struktur data terstandarisasi:

```rust
pub struct AudioSourceInfo {
    pub source_id: String,          // e.g. "src_sys", "src_spotify", "src_chrome"
    pub display_name: String,       // e.g. "Windows System Audio", "Spotify (Music Stream)"
    pub source_type: AudioSourceType, // SystemAudio | ApplicationProcess
    pub is_available: bool,
    pub process_name: Option<String>,
    pub process_id: Option<u32>,
    pub listener_count: u32,
}
```

Format JSON pada kontrol pesawat (TCP/TLS):
```json
{
  "source_id": "src_spotify",
  "display_name": "Spotify (Music Stream)",
  "source_type": "ApplicationProcess",
  "is_available": true,
  "process_name": "spotify.exe",
  "process_id": 8120,
  "listener_count": 2
}
```

---

## 4. Siklus Hidup Sumber & Mekanisme Fallback

### Masalah
Aplikasi yang sedang di-stream dapat ditutup oleh pengguna sewaktu-waktu (misal user menghentikan Spotify atau menutup browser Chrome).

### Strategi Penanganan
1. **Monitoring Proses**: Engine desktop secara periodik memantau status handle proses target.
2. **Deteksi Terminasi**: Ketika proses target berakhir:
   - Sumber audio ditandai `is_available = false`.
   - Desktop Session Manager memanggil `fallback_sessions_with_source(&dead_source_id, "src_sys")`.
   - Seluruh sesi perangkat yang sedang mendengarkan sumber tersebut dialihkan secara mulus ke `src_sys` (System Audio).
3. **Notifikasi Receiver**: Pesan kontrol `SourceChanged` dikirim ke perangkat Android terkait:
   ```json
   {
     "type": "SourceChanged",
     "payload": {
       "source_id": "src_sys",
       "display_name": "Windows System Audio (Default Output)",
       "fallback": true,
       "message": "Audio source 'Spotify' was closed. Automatically redirected to System Audio."
     }
   }
   ```
4. **Respon Android UI**: Receiver menampilkan banner peringatan oranye *"Source Re-routed"* yang dapat di-dismiss pengguna, tanpa memutus aliran audio atau buffer decoding.

---

## 5. Protokol Kontrol Routing

| Perintah | Deskripsi | Parameter |
|---|---|---|
| `GetSources` | Mengambil daftar semua sumber audio aktif & jumlah pendengar | `{}` |
| `SelectSource` | Memilih sumber audio untuk sesi pemanggil | `{"source_id": "src_..."}` |
| `SourceChanged` | Notifikasi bahwa sumber audio sesi telah berubah | `{"source_id": "...", "fallback": bool}` |

Routing dilakukan secara zero-interruption: jitter buffer penerima terus berjalan, dan decoder mengadaptasi stream baru secara mulus.
