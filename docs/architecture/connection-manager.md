# Connection Manager Architecture

## 1. Responsibilities
The `ConnectionManager` (located in `crates/networking/src/connection_manager.rs`) acts as the single source of truth for transport routing, multi-interface enumeration, failover arbitration, and network telemetry in AudioBridge.

### Core Functions:
- **Interface Enumeration:** Discovers active Wi-Fi, Ethernet, Loopback, and USB Tethering (RNDIS/NCM) interfaces.
- **Port Conflict Avoidance:** Guarantees atomic reservation of `:40120` (TCP control) and `:40121` (UDP media).
- **Transport Priority Policy:**
  - `PreferWifi`: Defaults to Wi-Fi; automatically falls back if wireless link degrades.
  - `PreferUsbTethering`: Promotes USB Tethering when plugged in; falls back to Wi-Fi on hot-unplug.
  - `PreferAdbTunnel`: Routes via ADB reverse tunnel when an authorized device is present.
  - `Manual`: Strict user-selected interface binding.
- **Session Continuity:** Preserves authenticated `ActiveSession` state and encryption keys across transport hops without duplicating sessions.

## 2. Interface Definition
```rust
pub struct ConnectionManager {
    active_mode: RwLock<ConnectionTransportMode>,
    policy: RwLock<TransportPriorityPolicy>,
    interfaces: RwLock<Vec<NetworkInterfaceInfo>>,
    usb_tether: RwLock<UsbTetheringStatus>,
    adb_status: RwLock<AdbDeviceStatus>,
    fallback_mode: RwLock<Option<ConnectionTransportMode>>,
    last_switch: RwLock<Instant>,
    active_reverse_tunnels: RwLock<HashMap<u16, u16>>,
}
```

## 3. UI Synchronization
The Connection Manager exposes state to Tauri desktop and Android Compose via reactive state flows:
- `get_connection_telemetry()`
- `get_network_interfaces()`
- `select_connection_mode(mode)`
