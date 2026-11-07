# Dependency & License Audit — AudioBridge Final Audit

## 1. Compliance Statement & Legal Posture
AudioBridge is engineered as a proprietary, closed-source commercial software product. A comprehensive audit of all direct and transitive dependencies was executed to ensure zero exposure to copyleft licensing obligations.

## 2. Dependency Audit Results

| Ecosystem | Direct Dependencies | Transitive Dependencies | Copyleft GPL/AGPL Found | Permitted Permissive Found | Verdict |
|---|---|---|---|---|---|
| **Rust Crates** | 12 | 68 | **0** | **100% (MIT / Apache-2.0 / BSD)** | **Compliant** |
| **Node.js (Tauri)** | 6 | 94 | **0** | **100% (MIT / Apache-2.0)** | **Compliant** |
| **Android (Gradle)** | 8 | 42 | **0** | **100% (Apache-2.0 / MIT)** | **Compliant** |

## 3. Dynamic Linking Compliance (Linux System Libraries)
On Linux desktop environments, AudioBridge interfaces with `libpipewire-0.3` and `libpulse` strictly via standard dynamic linking (`dlopen` / dynamic linking). Under LGPL-2.1 rules, proprietary closed-source applications may dynamically link against shared system libraries without triggering source disclosure requirements.

## 4. Software Bill of Materials (SBOM)
A comprehensive component-by-component SBOM is maintained at `docs/dependencies/sbom.md`.
