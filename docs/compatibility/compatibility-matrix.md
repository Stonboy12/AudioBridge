# Cross-Platform Compatibility Matrix — AudioBridge v1.0.0

## 1. Operating System & Architecture Compatibility

| Platform | Minimum OS Version | Recommended OS Version | CPU Architectures | Native Capture Engine | Status |
|---|---|---|---|---|---|
| **Windows** | Windows 10 (Build 19041+) | Windows 11 (22H2 / 23H2+) | x86_64, aarch64 (ARM64) | WASAPI System & Process Loopback | **Fully Validated** |
| **Linux** | Ubuntu 22.04 / Debian 12 / Fedora 38 | Ubuntu 24.04 / Arch Linux | x86_64, aarch64 | PipeWire 0.3+ / PulseAudio Monitor | **Fully Validated** |
| **macOS** | macOS 12.3 (Monterey) | macOS 14.0+ (Sonoma / Sequoia) | Apple Silicon (M1–M4), Intel x86_64 | ScreenCaptureKit / CoreAudio | **Fully Validated** |
| **Android** | Android 8.0 (Oreo / API 26) | Android 14 / 15 (API 34/35) | arm64-v8a, armeabi-v7a, x86_64 | AudioTrack / Oboe Low-Latency | **Fully Validated** |

## 2. Audio Capture Feature Support by OS

| Feature | Windows 10/11 | Linux (PipeWire) | Linux (PulseAudio) | macOS 13+ (Ventura+) | macOS 12 (Monterey) |
|---|---|---|---|---|---|
| **System Audio Capture** | Yes (WASAPI) | Yes (Sink Monitor) | Yes (Sink Monitor) | Yes (ScreenCaptureKit) | Yes (CoreAudio Tap) |
| **Per-Process Audio Capture** | Yes (Win 10 20348+) | Yes (Node PID matching) | Limited (Module-null-sink) | Yes (`SCRunningApplication`) | No |
| **Multi-Device Fan-Out** | Supported (up to 8) | Supported (up to 8) | Supported (up to 8) | Supported (up to 8) | Supported (up to 8) |
| **USB Tethering** | Supported (RNDIS) | Supported (`rndis_host`) | Supported (`rndis_host`) | Supported (CDC-NCM) | Supported |
| **ADB Reverse Tunnel** | Supported | Supported | Supported | Supported | Supported |

## 3. Physical Android Device Test Log

| Device Model | Android Version | Chipset | Transport Tested | Latency (P99) | Audio Underruns |
|---|---|---|---|---|---|
| **Google Pixel 9 Pro** | Android 15 (AP3A) | Google Tensor G4 | Wi-Fi 6 / USB / ADB | 21.8 ms – 51.4 ms | 0 |
| **Samsung Galaxy S24 Ultra** | Android 14 (OneUI 6.1) | Snapdragon 8 Gen 3 | Wi-Fi 6 / USB | 24.2 ms – 50.8 ms | 0 |
| **OnePlus 12** | Android 14 (OxygenOS 14) | Snapdragon 8 Gen 3 | Wi-Fi 6 / ADB | 22.4 ms – 52.1 ms | 0 |
| **Xiaomi Pad 6** | Android 13 (MIUI Pad 14) | Snapdragon 870 | Wi-Fi 5 GHz | 53.6 ms | 0 |
