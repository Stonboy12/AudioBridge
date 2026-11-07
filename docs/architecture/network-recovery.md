# Network Recovery & Seamless Transport Failover Architecture

## 1. Failure Scenarios & Recovery Matrix

| Trigger Event | Detection Window | Primary Action | Fallback Destination | Session Impact |
|---|---|---|---|---|
| **USB Cable Hot-Unplug** | < 100 ms | Carrier drop on `rndis0` / `usb0` | Active Wi-Fi/LAN interface | Retained (Token verified) |
| **ADB Daemon Crash / Kill** | < 250 ms | Socket EOF on `127.0.0.1:40120` | Wi-Fi / LAN | Retained |
| **Wi-Fi Router Reconnect / IP Change** | < 500 ms | Network change broadcast | Exponential backoff reconnect | Reconnected without duplicate |
| **Desktop Host Restart** | < 2.0 s | Keepalive timeout | Auto-reconnect on discovery | Fresh session with stored token |
| **Android Receiver App Sleep / Wake** | < 150 ms | OS wake lock / foreground event | Immediate socket resume | Instant unpause |

## 2. Exponential Backoff with Jitter
When link failure is detected, the reconnect loop executes a randomized exponential backoff policy:
```text
T_backoff = min(T_max, T_base * 2^attempt) + Uniform(-Jitter, +Jitter)
Where:
- T_base = 500 ms
- T_max = 8000 ms
- Jitter = +/- 150 ms
```
This prevents the "thundering herd" problem when multiple Android receivers simultaneously attempt reconnection after a router or desktop restart.

## 3. Session Deduping & Single-Session Invariant
To prevent duplicate streaming instances or memory leaks:
1. `SessionManager::terminate_by_client_id(client_id)` is invoked prior to establishing a replacement connection.
2. The previous media UDP socket is dropped and unbound before creating the new receiver socket.
3. The jitter buffer ring buffer is flushed cleanly without re-allocating memory chunks.
