# Software Bill of Materials (SBOM) — AudioBridge v1.0.0

## 1. Rust Desktop Core & Workspace Crates

| Component Name | Version | Supplier / Origin | License | Usage / Purpose |
|---|---|---|---|---|
| **audio-buffer** | 0.1.0 | AudioBridge Internal | Proprietary | Lock-free SPSC circular PCM buffer |
| **audio-capture** | 0.1.0 | AudioBridge Internal | Proprietary | Cross-platform WASAPI/PipeWire/SCK capture |
| **audio-codec** | 0.1.0 | AudioBridge Internal | Proprietary | Opus & PCM frame encoder/decoder wrapper |
| **audio-pipeline** | 0.1.0 | AudioBridge Internal | Proprietary | Real-time audio DSP and dispatch pipeline |
| **audio-types** | 0.1.0 | AudioBridge Internal | Proprietary | Audio profile and format data definitions |
| **networking** | 0.1.0 | AudioBridge Internal | Proprietary | UDP sender/receiver, pairing, connection manager |
| **streaming-protocol** | 0.1.0 | AudioBridge Internal | Proprietary | 30-byte binary ABRG datagram protocol |
| **diagnostics** | 0.1.0 | AudioBridge Internal | Proprietary | Real-time lockless metrics tracker |
| **config** | 0.1.0 | AudioBridge Internal | Proprietary | Atomic file-based configuration engine |
| **tokio** | 1.38.0 | Tokio Project | MIT | Asynchronous network runtime |
| **parking_lot** | 0.12.3 | Amanieu d'Antras | Apache-2.0 / MIT | High-performance sync primitives |
| **serde / serde_json** | 1.0.203 | dtolnay | Apache-2.0 / MIT | Configuration and RPC serialization |
| **thiserror** | 1.0.61 | dtolnay | Apache-2.0 / MIT | Idiomatic error handling |
| **rand** | 0.8.5 | Rand Project | Apache-2.0 / MIT | Cryptographic pairing token generation |
| **chrono** | 0.4.38 | Chrono Project | Apache-2.0 / MIT | UTC timestamp handling |

## 2. Desktop Frontend (Tauri 2 + React)

| Component Name | Version | Origin | License | Purpose |
|---|---|---|---|---|
| **@tauri-apps/api** | 2.0.0 | Tauri Programme | Apache-2.0 / MIT | Native Tauri IPC bridge |
| **react / react-dom** | 18.3.1 | Meta Platforms | MIT | Reactive desktop user interface |
| **typescript** | 5.4.5 | Microsoft Corp | Apache-2.0 | Type checking and compilation |
| **vite** | 5.2.11 | Evan You / Vite | MIT | Frontend bundler |

## 3. Android Mobile Receiver

| Component Name | Version | Origin | License | Purpose |
|---|---|---|---|---|
| **androidx.core:core-ktx** | 1.13.1 | Google LLC | Apache-2.0 | Android KTX foundation extensions |
| **androidx.compose.ui** | 1.6.7 | Google LLC | Apache-2.0 | Declarative UI framework |
| **androidx.compose.material3** | 1.2.1 | Google LLC | Apache-2.0 | Material Design 3 components |
| **androidx.lifecycle:lifecycle-viewmodel-compose** | 2.8.0 | Google LLC | Apache-2.0 | MVVM state management |
| **kotlinx-coroutines-android** | 1.8.1 | JetBrains s.r.o. | Apache-2.0 | Asynchronous coroutines runtime |
