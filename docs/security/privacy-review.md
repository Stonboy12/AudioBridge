# Privacy Review & Data Governance

## 1. Privacy First Architecture
AudioBridge is fundamentally engineered to preserve user privacy by ensuring that all captured sound and device identifiers remain strictly local to the user's private computer and mobile devices.

## 2. Key Privacy Guarantees
1. **No External Servers or Cloud Storage:** AudioBridge operates without servers, accounts, or telemetry endpoints.
2. **Local Loopback Only:** Audio capture APIs (`WASAPI Loopback`, `PipeWire Monitor`, `ScreenCaptureKit`) capture computer-generated playback only. Microphones and ambient acoustic inputs are completely excluded from the codebase.
3. **Ephemerality of Stream Data:** Audio packets reside solely in volatile RAM ring buffers for ~20-60 milliseconds before playback and are immediately overwritten. No audio recording or caching occurs.
4. **Configuration Minimality:** The configuration file (`audiobridge.toml`) stores only local port preferences, user-customized device nicknames, and paired device public identifiers.
