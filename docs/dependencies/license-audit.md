# Dependency License Audit & Closed-Source Distribution Compliance

## 1. Compliance Statement
AudioBridge is released as a proprietary, closed-source commercial software product. Every direct and transitive dependency has been audited to prevent the introduction of copyleft licenses (e.g. GNU GPLv2, GNU GPLv3, AGPL, or SSPL) that would impose source code disclosure requirements.

## 2. Permitted Permissive Licenses
All third-party libraries integrated into AudioBridge fall under strictly permissive licenses:
- **MIT License:** Unrestricted commercial distribution, modification, and bundling.
- **Apache License 2.0:** Permissive commercial licensing with express patent grants.
- **BSD 2-Clause / 3-Clause:** Permissive attribution-only distribution.

## 3. Dynamic Linking Considerations (Linux Audio Stack)
- `libpipewire-0.3` and `libpulse` are system libraries dynamically linked via standard POSIX C runtime (`dlopen` / dynamic linking). They are governed by LGPL-2.1, which explicitly permits proprietary closed-source applications to link against shared libraries (`.so`) without source disclosure.
- Windows WASAPI and macOS ScreenCaptureKit are proprietary operating system APIs provided directly by Microsoft and Apple under standard SDK agreements.

## 4. Conclusion
AudioBridge is **100% compliant** with closed-source commercial distribution standards across Windows, Linux, macOS, and Android.
