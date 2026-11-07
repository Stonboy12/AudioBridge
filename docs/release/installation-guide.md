# AudioBridge v1.0.0 — Installation Guide

## 1. System Requirements

### Desktop Host:
- **Windows:** Windows 10 (64-bit, 19041+) or Windows 11.
- **Linux:** Ubuntu 22.04+, Fedora 38+, Arch Linux, or Debian 12 with PipeWire or PulseAudio.
- **macOS:** macOS 12.3+ (macOS 13+ recommended for ScreenCaptureKit per-app capture).
- **Network:** Wi-Fi 802.11n/ac/ax on local router or direct USB cable.

### Android Receiver:
- **OS:** Android 8.0 (API 26) or higher.
- **Audio Output:** Built-in speakers, 3.5mm wired headphones, USB-C DAC, or Bluetooth A2DP.

## 2. Desktop Installation

### Windows:
1. Download `AudioBridge_1.0.0_x64-setup.exe`.
2. Run the installer and accept the standard installation directory.
3. If prompted by Windows Defender SmartScreen, click "More Info" -> "Run anyway" (pending EV Code Signing certificate).
4. Allow AudioBridge through Windows Firewall for private local networks.

### Linux:
1. For Debian/Ubuntu: `sudo dpkg -i audiobridge_1.0.0_amd64.deb`
2. For any Linux distribution: `chmod +x AudioBridge_1.0.0_amd64.AppImage && ./AudioBridge_1.0.0_amd64.AppImage`

### macOS:
1. Open `AudioBridge_1.0.0_universal.dmg`.
2. Drag `AudioBridge.app` into your `Applications` folder.
3. On first launch, navigate to **System Settings -> Privacy & Security -> Screen Recording** and grant permission for AudioBridge.

## 3. Android Installation
1. Transfer `app-debug.apk` or `AudioBridge-v1.0.0.apk` to your Android device.
2. Enable "Install Unknown Apps" in your browser or file manager.
3. Tap the APK file to install.
4. Launch **AudioBridge** — mDNS will automatically discover your desktop PC!
