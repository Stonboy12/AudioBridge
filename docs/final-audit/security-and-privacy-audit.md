# Security & Privacy Audit — AudioBridge Final Audit

## 1. Security Architecture Verification

### 1.1 Pairing Protocol (ADR-004)
- **Token Entropy:** 256-bit cryptographically secure random tokens generated via `rand::thread_rng()`.
- **Constant-Time Verification:** Token matching executes constant-time byte comparisons to prevent timing side-channel attacks.
- **PIN Expiry & Rate Limiting:** 6-digit confirmation PINs expire automatically after 180 seconds. A maximum of 3 incorrect attempts is enforced before invalidating the pairing challenge.
- **QR Fast Pairing:** Payload URI (`audiobridge://pair?host=...`) includes ephemeral challenge tokens that cannot be replayed.

### 1.2 Network Transport & Packet Validation
- **ABRG Protocol Validation:** Every UDP datagram is checked for:
  - Magic byte sequence: `0x41425247` (`ABRG`).
  - Protocol version: `1`.
  - Header boundary: `len >= 30 bytes`.
  - Payload length consistency: `payload_len == actual_bytes - 30`.
- **Replay Protection:** 64-bit monotonically increasing sequence numbers reject duplicate or out-of-order packets.
- **Localhost Isolation for ADB:** Reverse tunnel ports bind strictly to `127.0.0.1`. Full ADR-004 cryptographic token authentication remains mandatory over localhost.

### 1.3 Key Storage & Android Keystore Isolation
- On Android, device pairing tokens are encrypted with `AES-256-GCM` using keys stored in the hardware-backed **Android Keystore** (`AndroidKeyStore`), protecting keys against filesystem extraction even on rooted devices.

## 2. Privacy Audit & Data Governance
- **Zero Cloud Relay:** AudioBridge contains zero cloud backend dependencies, analytics libraries, or telemetry endpoints.
- **Render Loopback Only:** Audio capture engines bind strictly to render output endpoints. Hardware microphones and ambient inputs are excluded from the code.
- **Zero Audio Logging:** PCM and Opus sample payloads are never logged or stored to disk.
- **Atomic Local Persistence:** `audiobridge.toml` stores only local port configurations and user-approved device IDs.
