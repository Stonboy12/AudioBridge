# USB Tethering Network Architecture (RNDIS & CDC-NCM)

## 1. Operating Mechanism
When USB Tethering is toggled on an Android device connected to a desktop host (Windows, Linux, or macOS), the Android kernel exposes a virtual USB Network Interface Controller (NIC) to the host operating system:
- **Windows:** Remote NDIS (RNDIS) device driver.
- **Linux:** `rndis_host` or `cdc_ncm` kernel modules (yielding network interface `usb0` or `rndis0`).
- **macOS:** CDC-NCM / ECM class driver (or HoRNDIS for legacy versions).

The Android device acts as a DHCP server on a private subnet (typically `192.168.42.0/24` or `192.168.43.0/24`), assigning:
- Android Gateway IP: `192.168.42.129` (or `192.168.42.1`)
- Host Desktop IP: `192.168.42.128` (or `192.168.42.x`)

## 2. Detection & Route Selection
The AudioBridge Connection Manager continuously monitors local system network interfaces:
1. `ConnectionManager::refresh_interfaces()` detects the presence of an interface flagged with the default tethering subnet `192.168.42.x` or with driver description matching `RNDIS` / `USB Ethernet`.
2. When selected, the control plane and media datagram sockets bind to the tethered interface IP, bypassing Wi-Fi contention.

## 3. Performance Characteristics
- **Latency Reduction:** Bypasses 802.11 airtime queueing and CSMA/CA contention, dropping packet transit times from ~3-5 ms down to < 0.6 ms.
- **Zero RF Interference:** Immune to microwave, Bluetooth 2.4 GHz, or crowded apartment Wi-Fi congestion.
- **Simultaneous Charging:** The USB cable provides continuous power to the Android device, eliminating battery drain during prolonged streaming.

## 4. Limitations & Edge Cases
- **Carrier / OS Tethering Lock:** Some mobile operators or enterprise MDM policies disable the USB Tethering toggle switch.
- **Host Firewall Rules:** Certain Windows Defender or corporate firewall profiles classify new RNDIS interfaces as "Public Unidentified Networks" and may block inbound UDP traffic unless firewall exemptions for `:40121` are configured.
- **Hot-Unplug Recovery:** If the USB cable is physically unplugged, the RNDIS interface vanishes instantly. AudioBridge detects this carrier drop within 100ms and executes immediate failover to the active Wi-Fi interface without interrupting ongoing audio decoding.
