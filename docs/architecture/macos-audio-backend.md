# macOS Audio Backend Architecture (ScreenCaptureKit & CoreAudio)

## 1. Overview
On macOS 13.0+ (Ventura and later), Apple introduced **ScreenCaptureKit (SCK)** audio capture capabilities, allowing user-space applications to capture system audio and specific application audio streams without installing third-party virtual audio driver kexts/HAL plug-ins (such as BlackHole or Soundflower).

AudioBridge adopts ScreenCaptureKit as its primary macOS capture engine, with an optional CoreAudio HAL Tap fallback for legacy macOS versions.

## 2. ScreenCaptureKit Implementation

### 2.1 Configuration
```swift
let config = SCStreamConfiguration()
config.capturesAudio = true
config.sampleRate = 48000
config.channelCount = 2
config.excludesCurrentProcessAudio = true // Avoid capturing AudioBridge's own feedback

// Set minimal video overhead since we only need audio
config.width = 2
config.height = 2
config.minimumFrameInterval = CMTime(value: 1, timescale: 1)
```

### 2.2 System & Application Filtering (`SCContentFilter`)
- **System Audio Capture:**
  Created by filtering the main display (`SCDisplay`) with all applications included:
  ```swift
  let filter = SCContentFilter(display: mainDisplay, excludingApplications: [], exceptingWindows: [])
  ```
- **Per-Application Process Capture:**
  Created by filtering specifically for the target `SCRunningApplication`:
  ```swift
  let filter = SCContentFilter(display: mainDisplay, including: [targetApp], exceptingWindows: [])
  ```
  This captures pure audio from apps like Spotify, Chrome, or Final Cut Pro while excluding system notification chime sounds.

### 2.3 Audio Sample Buffer Output
Samples arrive via `SCStreamOutput.stream(_:didOutputSampleBuffer:of:)` with type `.audio`:
- Linear PCM IEEE Float32, 48kHz, 2 channels.
- Buffers are forwarded directly into `AudioPipeline` ring buffer via the Rust C-FFI / Objective-C bridge.

## 3. macOS Security, Permissions & Entitlements

### 3.1 TCC Screen & System Audio Recording Permission
Capturing system-wide audio on macOS requires user approval under **Privacy & Security -> Screen Recording**:
- AudioBridge calls `CGPreflightScreenCaptureAccess()` to check permission status.
- If denied or not yet prompted, `CGRequestScreenCaptureAccess()` opens the native macOS authorization prompt.
- The desktop UI displays a guidance banner if permission has not been granted.

### 3.2 Entitlements (`audiobridge.entitlements`)
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.device.audio-input</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
</dict>
</plist>
```
