# Android Debug Bridge (ADB) Security Review & Governance

## 1. Threat Model & Security Scope
Android Debug Bridge (ADB) is a powerful diagnostic subsystem that provides root-equivalent or developer-level debugging access to an Android operating system. Integrating ADB tunneling into a consumer audio product requires rigorous threat modeling and strict security boundaries.

## 2. Security Principles Enforced by AudioBridge

### 2.1 Zero Silent Activation
- **Constraint:** AudioBridge **never** attempts to enable USB Debugging programmatically or bypass OS prompts.
- **Enforcement:** USB Debugging can only be enabled by the user navigating to Android Settings -> Developer Options. If a device connects without authorization, AudioBridge flags it as `UNAUTHORIZED` and refuses to open reverse ports.

### 2.2 Localhost Isolation & Binding Boundary
- **Constraint:** Reverse tunnel ports must never be exposed to the public internet or external local network interfaces.
- **Enforcement:** `adb reverse` binds explicitly to the device loopback interface (`127.0.0.1`). No external device on the Wi-Fi or cellular network can reach the tunnel ports.

### 2.3 Protection of User Cryptographic Keys
- **Constraint:** The user's ADB RSA private key (`~/.android/adbkey`) must never be inspected, cached, copied, or transmitted.
- **Enforcement:** AudioBridge operates strictly as an unprivileged client communicating with the standard ADB daemon port (`127.0.0.1:5037`). It has zero access to filesystem key stores.

### 2.4 Cryptographic Authentication Remains Mandatory
- **Constraint:** Communication over localhost must not be assumed trusted.
- **Enforcement:** Even over ADB reverse tunnel, every session must complete the full 256-bit cryptographic token authentication (ADR-004) and 6-digit challenge confirmation. Malicious local apps on Android cannot hijack the audio stream.

### 2.5 Clean Teardown Guarantee
- **Constraint:** Abandoned reverse tunnel ports must not persist after session termination.
- **Enforcement:** `ConnectionManager` registers teardown hooks that issue `adb reverse --remove tcp:40120` and `tcp:40121` upon connection closure, disconnect, or application exit.
