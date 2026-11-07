# Changelog

All notable changes to **AudioBridge** are documented in this file.

## [1.0.0-rc1] - 2026-11-07

### Added
- **Cross-Platform Audio Capture Engine:**
  - Windows WASAPI Master Render Loopback & Process Loopback (`AUDIOCLIENT_ACTIVATION_PARAMS`).
  - Linux Native PipeWire 0.3 stream capture with automatic PulseAudio monitor sink fallback.
  - macOS ScreenCaptureKit (`SCStream`) with per-app audio filtering and CoreAudio fallback.
- **Smart Audio Routing & Multi-Device Fan-Out:**
  - Independent per-device source selection (Spotify, Chrome, Discord, System Audio).
  - $O(S \cdot P)$ shared circular capture and shared Opus encode threads feeding up to 8 concurrent Android devices.
  - Automatic graceful fallback to System Audio upon captured process termination.
- **Advanced Connectivity & Transport Manager:**
  - Standard 5 GHz Wi-Fi / LAN transport via mDNS (`_audiobridge._tcp`).
  - USB Tethering (RNDIS/NCM) delivering sub-25ms ultra-low jitter wired streaming.
  - ADB Reverse Tunnel (`adb reverse tcp:40120 & tcp:40121`) for restricted networks.
  - Seamless hot-unplug failover with exponential backoff and zero session duplication.
- **Cryptographic Security (ADR-004):**
  - 256-bit authentication tokens, 6-digit confirmation PINs, rate limiting, and 1-tap QR Code scanning.
  - Hardware-backed Android Keystore storage (`AES-256-GCM`).
- **Telemetry & Visualization:**
  - Real-time HTML5 Canvas waveform oscilloscope with dynamic peak and dBFS meters.
  - Wi-Fi jitter buffer monitoring and packet loss concealment (PLC) diagnostics.

### Changed
- Refactored `AudioBridge` branding across Desktop and Android apps.
- Updated `.gitignore` to prevent compilation cache bloat on GitHub.

### Security
- Conducted full vulnerability review; zero Critical or High issues found.
- Audited SBOM: 100% permissive MIT/Apache-2.0/BSD licenses.
