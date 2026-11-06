# AudioBridge — Native Bridge & C++/Rust JNI Architecture

**Document Version:** 1.0.0  
**Components:** Rust Core, C++ Oboe Audio Sink, Kotlin JNI Interface  
**Target ABIs:** `arm64-v8a`, `armeabi-v7a`, `x86_64`  
**License:** Proprietary / Closed Source  

---

## 1. Dual Audio Sink Strategy (ADR-002 Implementation)

AudioBridge implements a **Dual Audio Sink Architecture** on Android to combine peak performance with universal device compatibility:

```
+---------------------------------------------------------------------------------+
|                         NativeAudioBridge (Kotlin Singleton)                    |
+---------------------------------------+-----------------------------------------+
                                        | Check isNativeAvailable
            +---------------------------+---------------------------+
            | (Native .so Available)                                | (Fallback / Emulators)
            v                                                       v
+-------------------------------+                       +-------------------------------+
|   C++ Oboe / AAudio AudioSink |                       |   AudioTrackPlayer (Kotlin)   |
|   - LowLatency Exclusive Mode |                       |   - PERFORMANCE_MODE_LOW_LAT  |
|   - Real-time C++ Callback    |                       |   - USAGE_MEDIA Attributes    |
|   - Sub-15ms Hardware Latency |                       |   - Universal API 24+ Support |
+-------------------------------+                       +-------------------------------+
```

### Why Dual Architecture?
1. **Oboe / AAudio (Primary Production):** Directly taps into the Linux ALSA / hardware audio server bypassing framework binder IPC, minimizing latency down to 10–15ms on modern Android devices.
2. **AudioTrack (Universal Compatibility Fallback):** Guaranteed to function on 100% of Android devices, emulators, custom ROMs, and development test harnesses without native library compilation barriers.

---

## 2. JNI Interface Specification

The native layer exposes a minimal, leak-free C interface matching `com.example.NativeAudioBridge`:

| Native Function | Arguments | Return | Description |
|---|---|---|---|
| `nativeInitEngine` | `sampleRate: Int, channels: Int, targetDelayMs: Int` | `jlong (Handle)` | Instantiates native Rust receiver and jitter buffer |
| `nativeProcessPacket` | `handle: Long, packetData: ByteArray, offset: Int, length: Int` | `jint (Status)` | Validates header and enqueues payload |
| `nativeGetBufferDepthMs` | `handle: Long` | `jfloat` | Returns instantaneous jitter buffer depth in ms |
| `nativeGetUnderrunCount` | `handle: Long` | `jlong` | Returns hardware audio underrun count |
| `nativeDestroyEngine` | `handle: Long` | `void` | Deallocates native engine memory |

---

## 3. Real-Time Memory Management Across JNI Boundary

To eliminate Garbage Collector pauses:
1. **Handle Representation:** Native engine pointers are encapsulated as 64-bit integer handles (`jlong`), ensuring zero object wrapping on the JVM side.
2. **Direct Buffers:** Packet payloads pass through bounded byte buffers without intermediate heap copies.
3. **Graceful Degradation:** `NativeAudioBridge` uses a `try / catch (UnsatisfiedLinkError)` block inside its static initializer. If the `.so` binary is not bundled for a specific architecture, `isNativeAvailable` is set to `false`, and the app delegates seamlessly to `AudioTrackPlayer`.
