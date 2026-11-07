# Cross-Platform Desktop Audio Architecture (Windows, Linux, macOS)

## 1. Overview
AudioBridge is designed as a modular, high-performance real-time audio streamer. In Stage 4, native audio capture is extended from Windows WASAPI to Linux (PipeWire / PulseAudio) and macOS (ScreenCaptureKit / CoreAudio) while preserving identical downstream pipeline contracts (`AudioPipeline`, Opus/PCM encoding, and ABRG UDP datagram transport).

```text
+-----------------------------------------------------------------------------------+
|                            AudioBridge Native Capture                             |
+-----------------------------------------------------------------------------------+
        |                                   |                                   |
        v                                   v                                   v
+-----------------------+       +-----------------------+       +-----------------------+
|    Windows WASAPI     |       |    Linux PipeWire     |       |  macOS ScreenCapture  |
| - AUDCLNT_LOOPBACK    |       | - libpipewire-0.3     |       | - SCStream (macOS 13+)|
| - Process Loopback    |       | - PulseAudio Monitor  |       | - App Filter Streams  |
| - Min Latency: 10ms   |       | - Min Latency: 8ms    |       | - Min Latency: 12ms   |
+-----------------------+       +-----------------------+       +-----------------------+
        |                                   |                                   |
        +-----------------------------------+-----------------------------------+
                                            |
                                            v (Linear Float32, 48kHz, Stereo)
                    +-----------------------------------------------+
                    |             AudioCaptureEngine Trait          |
                    | - start(callback) -> Result<(), CaptureError> |
                    | - stop() -> Result<(), CaptureError>          |
                    | - available_devices() -> Vec<AudioDeviceInfo> |
                    +-----------------------------------------------+
                                            |
                                            v (10ms / 20ms Ring Buffer)
                    +-----------------------------------------------+
                    |          AudioPipeline & Multi-Device         |
                    | - Shared capture per audio source O(S)        |
                    | - Shared Opus encode per profile O(S * P)     |
                    | - Non-blocking UDP Fan-Out to Android clients |
                    +-----------------------------------------------+
```

## 2. Common Trait Contract: `AudioCaptureEngine`
All platform capture engines implement the identical thread-safe Rust trait:
```rust
pub trait AudioCaptureEngine: Send + Sync {
    fn start(&self, callback: AudioSampleCallback) -> Result<(), CaptureError>;
    fn stop(&self) -> Result<(), CaptureError>;
    fn is_running(&self) -> bool;
    fn active_device(&self) -> AudioDeviceInfo;
    fn available_devices(&self) -> Vec<AudioDeviceInfo>;
    fn frames_captured(&self) -> u64;
}
```

The sample callback receives interleaved `&[f32]` samples at 48,000 Hz stereo, normalized between `[-1.0, 1.0]`.

## 3. Platform Capability Matrix

| Feature | Windows (WASAPI) | Linux (PipeWire) | macOS (ScreenCaptureKit) |
|---|---|---|---|
| **Primary API** | WASAPI Loopback (`IAudioClient`) | `libpipewire-0.3` stream | `SCStream` (`ScreenCaptureKit.framework`) |
| **Fallback API** | WaveIn / DirectSound (legacy) | PulseAudio Simple / Monitor Sink | CoreAudio HAL Output Tap (`AudioServerPlugIn`) |
| **System Audio Capture** | Supported (All Win 10/11) | Supported (via Sink Monitor) | Supported (macOS 13.0+) |
| **Process Loopback** | Supported (Build 20348+ & Win 11) | Supported (PipeWire Node PID) | Supported (`SCRunningApplication`) |
| **Default Period** | 10 ms (480 frames) | 10 ms / 5.33 ms (PipeWire quantum) | 10 ms (480 frames) |
| **Hardware Permissions** | Granted (standard endpoint) | Granted (native socket / Portal) | TCC Screen & System Audio Recording |
| **Min Capture Latency** | ~10 ms | ~8 ms | ~12 ms |

## 4. Smart Audio Routing Adaptation
AudioBridge queries `PlatformCapabilityInfo::current()` on startup to discover OS features:
- If process loopback is not supported by the OS/distribution, the desktop UI automatically hides per-application routing dropdowns and defaults all connected Android receivers to System Audio.
- If a permission prompt is pending (e.g. macOS TCC or Linux Flatpak Portal), the UI alerts the user with platform-specific instructions without failing silently.
