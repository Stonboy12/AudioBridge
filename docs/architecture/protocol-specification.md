# AudioBridge — Protocol Specification (v1.0)

## 1. Scope and Compliance
This document specifies the wire format and message semantics for the AudioBridge Control and Media Planes according to **ADR-001**, **ADR-003**, and **ADR-004**.

---

## 2. Media Plane: Binary Audio Datagram Specification

Every audio packet sent over UDP Port 40121 begins with a mandatory, fixed-width **30-byte header** in network byte order (Big-Endian), followed immediately by the audio payload.

### 2.1 Header Structure (30 Bytes Exact)

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       'A'     |      'B'      |      'R'      |      'G'      | Magic (4B)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    Version    |     Flags     |     Codec     |   Reserved    | Ver/Flags/Codec (4B)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                           Stream ID                           | Stream ID (4B)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Sequence Number (High 32)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ Sequence (8B)
|                    Sequence Number (Low 32)                   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                 Monotonic Timestamp (High 32)                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ Timestamp (8B)
|                 Monotonic Timestamp (Low 32)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|         Payload Length        |                                 Length (2B)
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### 2.2 Field Definitions
1. **Magic (`0..3`):** 4 bytes ASCII `0x41 0x42 0x52 0x47` (`"ABRG"`). Packets missing this magic byte sequence are dropped immediately.
2. **Version (`4`):** `uint8` protocol version (`0x01`).
3. **Flags (`5`):** `uint8` bitmask:
   - Bit 0: `FLAG_KEYFRAME` (`0x01`) — Indicates standalone frame / sync point.
   - Bit 1: `FLAG_FEC_PRESENT` (`0x02`) — In-band forward error correction included.
   - Bit 2: `FLAG_DISCONTINUITY` (`0x04`) — Audio pipeline reset or silence gap.
4. **Codec (`6`):** `uint8` codec identifier:
   - `0x00`: `LinearPCM_S16LE` (48,000 Hz, 2 channels, 16-bit signed).
   - `0x01`: `Opus` (Interactive audio, variable bitrate).
5. **Reserved (`7`):** `uint8` reserved for future alignment (`0x00`).
6. **Stream ID (`8..11`):** `uint32` monotonic identifier assigned at session initiation.
7. **Sequence Number (`12..19`):** `uint64` contiguous packet index starting at `0` or randomized offset.
8. **Monotonic Timestamp (`20..27`):** `uint64` microsecond timestamp from system monotonic reference clock.
9. **Payload Length (`28..29`):** `uint16` size in bytes of the audio data immediately following the header.

---

## 3. Control Plane: JSON Message Schemas

Framed TCP stream on Port 40120. Length prefix: 4 bytes big-endian unsigned integer.

### 3.1 Initial Handshake
#### Client -> Server: `Hello`
```json
{
  "type": "Hello",
  "payload": {
    "client_id": "android_rx_pixel9_01",
    "client_name": "Google Pixel (Android Receiver)",
    "protocol_version": 1
  }
}
```

#### Server -> Client: `HelloResponse`
```json
{
  "type": "HelloResponse",
  "payload": {
    "server_name": "Desktop-WASAPI",
    "server_version": "0.2.0",
    "supported_codecs": ["Opus", "PCM"],
    "requires_pairing": true
  }
}
```

---

### 3.2 Secure Pairing Workflow (ADR-004)
#### Client -> Server: `PairRequest`
```json
{
  "type": "PairRequest",
  "payload": {
    "client_id": "android_rx_pixel9_01",
    "client_name": "Google Pixel (Android Receiver)"
  }
}
```

#### Server -> Client: `PairChallenge`
```json
{
  "type": "PairChallenge",
  "payload": {
    "challenge_id": "ch_7a9f82c410",
    "pairing_code": "849 201",
    "expires_in_secs": 180,
    "max_attempts": 3
  }
}
```

#### Client -> Server: `PairVerify`
```json
{
  "type": "PairVerify",
  "payload": {
    "challenge_id": "ch_7a9f82c410",
    "code": "849201"
  }
}
```

#### Server -> Client: `PairResponse`
```json
{
  "type": "PairResponse",
  "payload": {
    "success": true,
    "auth_token": "a4f89d38c10e64b219582d90ef3247bb71840294821a0937b420f188374d9201",
    "message": "Device successfully paired and trusted"
  }
}
```

---

### 3.3 Session Authentication & Streaming
#### Client -> Server: `Authenticate`
```json
{
  "type": "Authenticate",
  "payload": {
    "client_id": "android_rx_pixel9_01",
    "auth_token": "a4f89d38c10e64b219582d90ef3247bb71840294821a0937b420f188374d9201"
  }
}
```

#### Server -> Client: `AuthResponse`
```json
{
  "type": "AuthResponse",
  "payload": {
    "success": true,
    "session_id": "sess_810932fb91",
    "message": "Authenticated successfully"
  }
}
```

#### Client -> Server: `StartStream`
```json
{
  "type": "StartStream",
  "payload": {
    "udp_listen_port": 40121,
    "preferred_codec": "Opus",
    "bitrate_kbps": 128
  }
}
```

#### Server -> Client: `StreamStarted`
```json
{
  "type": "StreamStarted",
  "payload": {
    "session_id": "sess_810932fb91",
    "stream_id": 1001,
    "codec": "Opus",
    "sample_rate": 48000,
    "channels": 2,
    "frame_duration_ms": 20
  }
}
```

---

### 3.4 Telemetry & Keep-Alive
#### Client <-> Server: `Ping` / `Pong`
```json
{
  "type": "Ping",
  "payload": {
    "client_timestamp_us": 1730799482100450
  }
}
```
```json
{
  "type": "Pong",
  "payload": {
    "client_timestamp_us": 1730799482100450,
    "server_timestamp_us": 1730799482101250
  }
}
```

---

### 3.5 Revocation & Disconnect
#### Client -> Server or Server -> Client: `RevokeDevice`
```json
{
  "type": "RevokeDevice",
  "payload": {
    "client_id": "android_rx_pixel9_01"
  }
}
```

#### Server -> Client: `DeviceRevoked`
```json
{
  "type": "DeviceRevoked",
  "payload": {
    "client_id": "android_rx_pixel9_01",
    "reason": "Revoked by host administrator"
  }
}
```
