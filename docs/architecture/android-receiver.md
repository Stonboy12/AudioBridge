# AudioBridge — Android Receiver Architecture

**Document Version:** 1.0.0  
**Target Platform:** Android 7.0+ (API 24 to API 36)  
**Architecture Pattern:** Clean Architecture / MVVM (Model-View-ViewModel) + Android Services  
**License Compliance:** Proprietary / Closed Source  

---

## 1. System Overview

The **AudioBridge Android Receiver** is the mobile endpoint responsible for receiving, buffering, decoding, and rendering high-fidelity PC audio streams over local Wi-Fi with ultra-low latency (<60ms).

```
+---------------------------------------------------------------------------------+
|                               Jetpack Compose UI                                |
|  - Dashboard / Receiver Hero   - Real-time Waveform   - Diagnostics & Telemetry |
|  - Onboarding & Pairing Dialog - Smart Audio Routing  - Validation Suite        |
+---------------------------------------+-----------------------------------------+
                                        | StateFlow / User Intent
                                        v
+---------------------------------------------------------------------------------+
|                       ReceiverViewModel (State Orchestrator)                     |
|  - Connection Lifecycle        - Jitter Buffer Tuning  - Diagnostic Telemetry   |
|  - Audio Focus Coordinator     - Routing Detector      - Volume & Ducking Control|
+-------------------+-------------------+-------------------+---------------------+
                    |                   |                   |
                    v                   v                   v
+-------------------+---+   +-----------+-----------+   +---+---------------------+
| AudioBridgeService     |   | UdpMediaReceiver      |   | AudioFocusHelper        |
| - Foreground Service  |   | - UDP Socket (40121)  |   | - USAGE_MEDIA           |
| - type="mediaPlayback"|   | - 30-Byte ABRG Header |   | - Ducking to 20%        |
| - Partial WakeLock    |   | - Packet Validation   |   | - Noisy Broadcast Rx    |
+-------------------+---+   +-----------+-----------+   +---+---------------------+
                    |                   |                   |
                    +-------------------+-------------------+
                                        |
                                        v
+---------------------------------------------------------------------------------+
|                              Audio Pipeline Core                                |
|  +--------------------------------+   +--------------------------------------+  |
|  |     JitterBufferManager        |   |       AudioTrackPlayer (Low-Latency) |  |
|  | - RFC 3550 Transit Jitter Filter|   | - PERFORMANCE_MODE_LOW_LATENCY       |  |
|  | - Out-of-Order Packet Sorting  |   | - Zero-Allocation Real-Time Audio    |  |
|  | - Packet Loss Concealment (PLC)|   | - C++ Oboe / AAudio Native Sink      |  |
|  +--------------------------------+   +--------------------------------------+  |
+---------------------------------------------------------------------------------+
```

---

## 2. Component Specifications

### 2.1 UI Layer (Jetpack Compose)
- **Declarative Reactive UI:** Fully reactive Compose hierarchy observing `ReceiverUiState` from `ReceiverViewModel`.
- **TopAppBar & System Sync:** Shows real-time connection state indicator (Standby, Connecting, Streaming, Reconnecting, Error).
- **Hero Card & Visualizer:** Displays PC transmitter metadata (`IP:Port`), active stream format, live waveform visualizer, audio output route, and ducking status.
- **Diagnostics Screen:** 6 key telemetry cards + Jitter buffer progress indicator, loss rate, underrun counter, packets received/lost/concealed/dropped/malformed.
- **Validation Suite:** Integrated automated test runner validating experiments EXP-1 through EXP-8.
- **Edge-to-Edge:** Native `enableEdgeToEdge()` with Material 3 styling and responsive density.

### 2.2 Lifecycle & Background Execution (`AudioBridgeReceiverService`)
- **Android 14+ Compliance:** Declared with `android:foregroundServiceType="mediaPlayback"`.
- **Foreground Notification:** Ongoing notification with playback status, active PC name, and direct "Disconnect" pending intent action.
- **CPU WakeLock:** Acquires a `PowerManager.PARTIAL_WAKE_LOCK` with an automatic 60-minute safety timeout to ensure the CPU does not throttle or suspend audio processing when the device screen is turned off.

### 2.3 Network Media Plane (`UdpMediaReceiver`)
- **Local Port:** Default UDP port 40121.
- **Socket Buffer:** Configured with 256 KB receive buffer to absorb network transmission spikes without packet dropping.
- **Binary Header Validation:** Enforces the 30-byte `ABRG` header:
  - Magic `0x41425247`
  - Version `0x01`
  - Stream ID, Session ID, Sequence Number (u64), Timestamp (u64 microseconds), Payload Length (u16).
  - Malformed or truncated packets are discarded immediately before entering the buffer.

### 2.4 Audio Focus & Routing (`AudioFocusHelper` & `AudioNoisyReceiver`)
- **Media Attributes:** Configured with `AudioAttributes.USAGE_MEDIA` and `CONTENT_TYPE_MUSIC`.
- **Ducking:** During transient notifications (e.g., GPS turn prompt or incoming message sound), the volume smoothly drops to 20% (`isDucked = true`) and restores upon focus gain.
- **Transient Pause:** When a phone call rings, audio stream pauses immediately.
- **Permanent Loss:** If another media player starts playback, the stream safely stops.
- **Becoming Noisy:** Intercepts `ACTION_AUDIO_BECOMING_NOISY`. If headphones or Bluetooth are disconnected, streaming pauses automatically to avoid loudspeaker blast.

---

## 3. Real-Time Threading Model

To ensure zero stutter and prevent Android Choreographer skips:
1. **Network Thread:** Dedicated high-priority thread running `socket.receive()`. Writes directly into `JitterBufferManager`.
2. **Audio Render Thread:** Highest priority (`Thread.MAX_PRIORITY`), reads packet from jitter buffer, applies Opus decompression or PLC comfort interpolation, and writes to `AudioTrack` via non-blocking `WRITE_NON_BLOCKING`.
3. **UI Thread:** Main thread collects `StateFlow` updates throttled to 500ms intervals for telemetry, ensuring zero UI overhead on audio performance.
