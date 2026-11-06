# AudioBridge — Multi-Device Session Management & Isolation

## 1. Objective
AudioBridge supports streaming desktop audio to multiple mobile receivers on the same LAN (e.g., dual headphones for co-watching, or streaming to a smartphone and a tablet simultaneously). 

The Session Management layer guarantees **complete state and fault isolation**:
- Disconnecting, crashing, or backgrounding one client does not interrupt or glitch active streams to other connected devices.
- Network congestion or Wi-Fi packet drops on one receiver never degrade the buffer depth or bitrate of other receivers.

---

## 2. Session Architecture & Model

### 2.1 Active Session Data Model (`crates/networking/src/session.rs`)
```rust
pub struct ActiveSession {
    pub session_id: String,       // 128-bit hex UUID
    pub client_id: String,        // Unique device hardware identifier
    pub client_name: String,      // Human-readable client label
    pub client_ip: IpAddr,        // Client IP address
    pub media_port: u16,          // UDP listening port of receiver
    pub is_streaming: bool,       // Active streaming state
    pub last_heartbeat: Instant,  // Monotonic time of last Ping/Pong
    pub started_at: Instant,      // Session creation time
}
```

---

## 3. Session State Machine

```
   [TCP Connected]
          |
          v
    +------------+
    | UNVERIFIED |----(Auth Failed / Timeout)----> [Closed]
    +------------+
          |
    (Authenticate OK)
          |
          v
    +------------+
    | AUTHENTIC  |----(Disconnect)---------------> [Closed]
    +------------+
          |
    (StartStream)
          |
          v
    +------------+
    | STREAMING  |<===(Ping / Pong Keep-Alive)
    +------------+
          |
     +----+-------------------+
     |                        |
(StopStream)          (Heartbeat Timeout > 10s / TCP Drop)
     |                        |
     v                        v
+------------+          +------------+
| AUTHENTIC  |          | RECONNECT  |
+------------+          +------------+
                              |
                     (Reconnection Succeeds)
                              |
                              v
                        +------------+
                        | STREAMING  |
                        +------------+
```

---

## 4. Multi-Client Fanout & Performance Isolation
1. **Zero-Copy Audio Dispatch:** The desktop WASAPI loopback capture thread delivers PCM frames to an audio pipeline. The encoded Opus packet is dispatched to a list of active UDP client endpoints concurrently:
   ```rust
   for session in session_manager.get_streaming_sessions() {
       let target = SocketAddr::new(session.client_ip, session.media_port);
       let _ = udp_socket.send_to(&datagram, target).await;
   }
   ```
2. **Independent UDP Transmit Queues:** OS-level non-blocking UDP sockets ensure that a slow or lagging client does not block sends to other clients.
3. **Heartbeat & Stale Pruning:** A periodic sweep (`cleanup_stale(Duration::from_secs(10))`) detects dropped connections and reclaims session state without leaking memory or OS sockets.
