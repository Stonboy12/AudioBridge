# AudioBridge — Pairing Security & Device Identity (ADR-004)

## 1. Overview
AudioBridge is a closed-source, local-first system designed to stream confidential desktop audio to mobile devices over private LANs. 

Because audio streams frequently contain private communications (voice calls, confidential meetings, private entertainment), unauthorized devices on the local Wi-Fi network must **never** be permitted to listen to audio packets without explicit user consent.

---

## 2. Threat Model & Security Objectives

| Threat | Impact | Countermeasure |
| :--- | :--- | :--- |
| **Unauthorized LAN Eavesdropping** | Third party on same Wi-Fi intercepts stream | Mandatory control plane pairing + authentication gate |
| **Brute-Force Code Guessing** | Attacker iterates 6-digit codes | 3 failed attempts lockout + 180s challenge timeout |
| **Replay Attacks** | Re-sending captured control packets | 256-bit cryptographically secure pseudorandom tokens (`CSPRNG`) |
| **Stolen Token / Compromised Phone** | Rogue receiver retains persistent access | Desktop and Android one-click device revocation |
| **Man-in-the-Middle (MITM)** | Attacker intercepts initial pairing | Out-of-band visual verification of 6-digit code on desktop UI |

---

## 3. Four-Step Pairing Protocol

```
    Android Receiver                          Desktop Transmitter
           |                                           |
           | 1. PairRequest(client_id, name)           |
           |------------------------------------------>|
           |                                           | Generates 6-digit PIN
           |                                           | (e.g. "849 201")
           |                                           | Starts 180s timer
           | 2. PairChallenge(challenge_id, code)      | Displays modal on Desktop UI
           |<------------------------------------------|
           |                                           |
    [User checks Desktop UI PIN]                       |
           |                                           |
           | 3. PairVerify(challenge_id, code)         |
           |------------------------------------------>| Validates PIN & Attempt Count
           |                                           | (Max 3 attempts)
           |                                           |
           |                                           | If match: Generates 256-bit Token
           |                                           | Stores in AppConfig.trusted_devices
           | 4. PairResponse(success, auth_token)      |
           |<------------------------------------------|
           |                                           |
    [Saves token in local secure store]                |
           |                                           |
           | 5. Authenticate(client_id, auth_token)    |
           |------------------------------------------>| Validates token in constant time
           | 6. AuthResponse(success, session_id)      |
           |<------------------------------------------|
```

---

## 4. Security Implementation Details

### 4.1 Cryptographic Token Generation
Tokens are generated using OS-level cryptographically secure entropy (`ring::rand::SystemRandom` in Rust, `java.security.SecureRandom` in Kotlin).
- **Entropy:** 32 bytes (256 bits).
- **Encoding:** Hexadecimal ASCII string (64 characters).
- **Collision Probability:** $< 10^{-77}$.

### 4.2 Rate Limiting & Lockout
1. **Challenge Timeout:** Each pairing challenge expires in exactly **180 seconds** (3 minutes). Expired challenges are deleted immediately from memory.
2. **Attempt Counter:** A maximum of **3 attempts** is enforced per challenge.
3. **Instant Invalidation:** Upon the 3rd failed attempt, the challenge is purged immediately and the server returns `MaxAttemptsExceeded`. Further attempts with that `challenge_id` are rejected with HTTP/JSON error.

### 4.3 Desktop Manual Approval
In addition to the PIN verification, the Desktop UI displays an interactive dialog showing:
- Client Name (e.g., `"Google Pixel (Android Receiver)"`)
- Client Hardware ID
- Client IP Address
- Generated PIN

The desktop user can click **Approve** or **Reject** at any time. Clicking "Reject" immediately aborts the pairing challenge.

### 4.4 Token Revocation
Both sides can revoke pairing at any time:
- **Desktop:** The user opens the "Devices" tab, locates the device in "Trusted Devices", and clicks **Revoke**. The token is removed from `AppConfig` and any active streaming sessions associated with that `client_id` are disconnected instantly.
- **Android:** The user opens "Settings", locates the transmitter in "Trusted Transmitters", and clicks **Forget / Revoke**. The local token is deleted, and `RevokeDevice` is dispatched to the desktop transmitter.
