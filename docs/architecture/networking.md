# AudioBridge — Dual-Plane Networking Architecture

## 1. Executive Summary
AudioBridge employs a **Dual-Plane Networking Architecture** designed specifically for real-time, low-latency audio transmission over local wireless networks (Wi-Fi 5/6/6E/7 and Ethernet). 

To ensure maximum throughput, minimal latency, and robust session control, the architecture completely decouples:
1. **Control Plane (Reliable / State-Oriented):** Framed JSON messages over stream-oriented TCP (Port 40120) for device discovery handshake, mutual authentication, pairing, parameter negotiation, session lifecycle, and keep-alive heartbeats.
2. **Media Plane (Unreliable / Real-Time Datagram):** Low-overhead binary datagrams over UDP (Port 40121) transporting Opus-encoded or raw PCM audio frames with an exact 30-byte header.

```
+-------------------------------------------------------------------------+
|                              LOCAL NETWORK                              |
+-------------------------------------------------------------------------+
       |                                                    ^
       | mDNS Discovery (_audiobridge._tcp, Port 40120)    |
       v                                                    |
+-------------------+                              +-------------------+
|                   |  TCP Port 40120 Control      |                   |
|                   |  [Framed JSON Control Plane] |                   |
|                   |<============================>|                   |
|  Desktop Sender   |                              |  Android Receiver |
|  (Windows / Linux)|  UDP Port 40121 Media        |  (Google Pixel /  |
|                   |  [30-byte Header + Audio]    |   Samsung Galaxy) |
|                   |----------------------------->|                   |
+-------------------+                              +-------------------+
```

---

## 2. Port Allocation & Protocol Matrix

| Channel | Default Port | Protocol | Reliability | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **Discovery** | 5353 | mDNS / UDP | Best Effort | Service broadcast (`_audiobridge._tcp.local.`) |
| **Control Plane** | 40120 | TCP | Reliable / Ordered | Pairing, Auth, Session Control, Heartbeats |
| **Media Plane** | 40121 | UDP | Unreliable / Real-Time | Opus & PCM Audio Datagrams |

---

## 3. Control Plane Architecture

### 3.1 Connection Lifecycle
The Control Plane operates as an asynchronous TCP server on the desktop (`ControlPlaneServer`) and a coroutine-backed TCP client on Android (`ControlPlaneClient`).

1. **Framing Mechanism:** Every control message is framed with a 4-byte big-endian unsigned integer indicating the length of the UTF-8 JSON payload, followed by the payload itself:
   ```
   +------------------------------------+--------------------------------+
   | Payload Length (4 bytes, Big-Endian) | UTF-8 JSON String (N bytes)   |
   +------------------------------------+--------------------------------+
   ```
2. **Message Envelope:** All control messages follow a standardized JSON envelope:
   ```json
   {
     "type": "MessageType",
     "payload": { ... }
   }
   ```
3. **Session Binding:** Once authenticated, a unique 128-bit hex UUID `session_id` is bound to the TCP socket and media stream.

---

## 4. Media Plane Architecture

### 4.1 Datagram Design Principles
- **No Head-of-Line Blocking:** Audio frames are transmitted over UDP so that a lost packet never stalls or delays subsequent packets.
- **Fixed-Size Minimal Header:** Exactly 30 bytes in length, parsed with zero heap allocation in both Rust (`bytes::Buf`) and Kotlin (`java.nio.ByteBuffer`).
- **Monotonic Timestamps:** Microsecond-resolution timestamps (`timestamp_us`) derived from system monotonic clocks enable RFC 3550 jitter calculation and transit delay tracking.
- **Stream Identification:** A 32-bit `stream_id` ensures that packets from old, terminated, or restarted streams are discarded immediately.

---

## 5. Security & Isolation Boundaries
1. **Local-First Boundary:** Transmissions are strictly bound to private IP subnets (RFC 1918). External cloud relays or STUN/TURN servers are completely prohibited.
2. **Per-Client Isolation:** Multiple Android receivers can connect to a single desktop simultaneously. Disconnecting or crashing one client does not disrupt the streams of other active clients.
3. **Hardware Fallback:** If mDNS multicast is filtered by network switches or routers (common on enterprise or guest Wi-Fi), the client falls back seamlessly to direct IP:Port unicast connection.
