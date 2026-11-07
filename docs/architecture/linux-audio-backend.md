# Linux Audio Backend Architecture (PipeWire & PulseAudio)

## 1. Overview
The Linux backend in AudioBridge leverages modern **PipeWire** graph-based multimedia routing as its primary capture path, providing direct stream tap capabilities and low latency without requiring virtual ALSA loopback kernel modules (`snd-aloop`).

For legacy Linux installations without PipeWire (e.g. older enterprise LTS distros), AudioBridge seamlessly falls back to **PulseAudio** monitor sink recording (`.monitor`).

## 2. PipeWire Implementation Details

### 2.1 Stream Initialization
```c
// PipeWire native stream creation
struct pw_properties *props = pw_properties_new(
    PW_KEY_MEDIA_TYPE, "Audio",
    PW_KEY_MEDIA_CATEGORY, "Capture",
    PW_KEY_MEDIA_ROLE, "Music",
    PW_KEY_NODE_NAME, "AudioBridge-System-Loopback",
    PW_KEY_STREAM_CAPTURE_SINK, "true",
    NULL
);

struct pw_stream *stream = pw_stream_new(core, "AudioBridge Streamer", props);
```

### 2.2 System Audio Capture
- In PipeWire, capturing default output involves binding a recording stream to the target output node or passing `PW_KEY_STREAM_CAPTURE_SINK = true`.
- PipeWire routes mixed PCM samples from the output sink directly to our stream buffer with zero resampling overhead when configured for 48,000 Hz F32 stereo.

### 2.3 Per-Application Capture (Process Loopback)
- PipeWire exposes all active audio nodes in the system graph with metadata properties:
  - `application.process.id`: The process ID (PID) of the application.
  - `application.name`: The executable name (e.g. `chrome`, `spotify`).
- By setting `target.object = <node_id>` on our recording stream, AudioBridge isolates and captures audio exclusively from that process, implementing process loopback natively on Linux.

## 3. PulseAudio Fallback
When PipeWire is absent (`/run/user/1000/pipewire-0` does not exist), AudioBridge queries PulseAudio for the default sink's monitor source:
```text
default_sink = pactl get-default-sink (e.g. alsa_output.pci-0000_00_1f.3.analog-stereo)
monitor_source = default_sink + ".monitor"
```
The monitor source acts as a continuous loopback stream that feeds directly into AudioBridge's PCM ring buffer.

## 4. Permissions & Sandboxing (Flatpak / Snap)
- When running inside Flatpak or Snap, direct access to the PipeWire daemon socket may be restricted.
- AudioBridge utilizes the **XDG Desktop Portal** (`org.freedesktop.portal.ScreenCast` or `PipeWire` portal) to request stream access with the `Audio` capture flag enabled.
- On standard distributions (Ubuntu, Fedora, Arch, Debian), user-level access to `/run/user/<uid>/pipewire-0` is granted by default without special root permissions.
