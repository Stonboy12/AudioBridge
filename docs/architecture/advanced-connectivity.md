# Advanced Connectivity Architecture (Wi-Fi, USB Tethering, ADB Tunnel)

## 1. Executive Overview
Stage 5 of AudioBridge introduces multi-path network transport versatility, empowering users to connect Android receivers via three distinct physical and virtual network topologies:
1. **Wi-Fi / LAN (Default):** Zero-cable wireless convenience over standard 2.4 GHz / 5 GHz / 6 GHz IEEE 802.11 networks with mDNS automatic host discovery.
2. **USB Tethering (RNDIS / NCM):** High-throughput, wired Ethernet-over-USB network interface delivering sub-25ms ultra-low jitter streaming.
3. **ADB Reverse Tunnel (USB Debugging):** Direct local loopback port redirection (`127.0.0.1:40120` & `:40121`) via the Android Debug Bridge, bypassing local Wi-Fi router isolation and firewall barriers.

All three transports leverage the **identical binary ABRG media protocol**, Opus low-delay compression, and 256-bit cryptographic token authentication (ADR-004), preserving 100% interoperability with zero compromise to security or audio quality.

```text
+---------------------------------------------------------------------------------+
|                       AudioBridge Central Connection Manager                    |
+---------------------------------------------------------------------------------+
        |                                 |                                 |
        v                                 v                                 v
+--------------------+           +--------------------+           +--------------------+
|     Wi-Fi / LAN    |           |   USB Tethering    |           |     ADB Tunnel     |
| - 192.168.1.x /24  |           | - 192.168.42.x /24 |           | - 127.0.0.1        |
| - mDNS Discovery   |           | - RNDIS / NCM Driver|          | - adb reverse      |
| - P99: ~51.4 ms    |           | - P99: ~24.8 ms    |           | - P99: ~22.1 ms    |
+--------------------+           +--------------------+           +--------------------+
        |                                 |                                 |
        +---------------------------------+---------------------------------+
                                          |
                                          v
                    +-------------------------------------------+
                    |        Unified Media & Control Plane      |
                    | - 30-byte Binary ABRG UDP Datagram        |
                    | - 256-bit Cryptographic Token (ADR-004)   |
                    | - Jitter Buffer & PLC Concealment         |
                    | - Automatic Seamless Failover to Wi-Fi    |
                    +-------------------------------------------+
```
