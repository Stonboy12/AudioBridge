use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use audio_types::{AudioProfile, AudioSourceInfo};
use streaming_protocol::{ControlMessage, PROTOCOL_VERSION};

use crate::pairing::PairingManager;
use crate::session::SessionManager;

pub struct ServerConfig {
    pub server_id: String,
    pub server_name: String,
    pub control_port: u16,
    pub media_port: u16,
    pub trusted_tokens: Arc<RwLock<std::collections::HashMap<String, String>>>, // client_id -> auth_token
}

pub struct ControlPlaneServer {
    config: ServerConfig,
    pairing_manager: Arc<PairingManager>,
    session_manager: Arc<SessionManager>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ControlPlaneServer {
    pub fn new(
        config: ServerConfig,
        pairing_manager: Arc<PairingManager>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            pairing_manager,
            session_manager,
            shutdown_tx,
        }
    }

    pub async fn run(self: Arc<Self>, bind_addr: SocketAddr) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(bind_addr).await?;
        info!("AudioBridge Control Plane Server listening on {}", bind_addr);

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((stream, peer_addr)) => {
                            let server = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = server.handle_client(stream, peer_addr).await {
                                    warn!("Client handler error for {}: {}", peer_addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Error accepting TCP connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Control Plane Server received shutdown signal");
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    async fn handle_client(&self, stream: TcpStream, peer_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        let mut authenticated_client_id: Option<String> = None;
        let mut active_session_id: Option<u32> = None;

        while let Some(line) = lines.next_line().await? {
            let msg: ControlMessage = match serde_json::from_str(&line) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Malformed JSON message from {}: {}", peer_addr, e);
                    continue;
                }
            };

            let response: Option<ControlMessage> = match msg {
                ControlMessage::Hello { client_id, client_name: _, client_version: _, protocol_version: _ } => {
                    let is_trusted = self.config.trusted_tokens.read().contains_key(&client_id);
                    Some(ControlMessage::HelloAck {
                        server_name: self.config.server_name.clone(),
                        server_version: "0.1.0".to_string(),
                        server_id: self.config.server_id.clone(),
                        requires_pairing: !is_trusted,
                    })
                }

                ControlMessage::Authenticate { client_id, auth_token } => {
                    let trusted = {
                        let tokens = self.config.trusted_tokens.read();
                        tokens.get(&client_id).map(|t| t == &auth_token).unwrap_or(false)
                    };

                    if trusted {
                        authenticated_client_id = Some(client_id.clone());
                        let session = self.session_manager.create_session(
                            client_id.clone(),
                            format!("Client-{}", &client_id[..client_id.len().min(8)]),
                            peer_addr,
                        );
                        let sid = session.read().session_id;
                        active_session_id = Some(sid);
                        Some(ControlMessage::AuthResponse {
                            success: true,
                            session_id: Some(sid),
                            error_message: None,
                        })
                    } else {
                        Some(ControlMessage::AuthResponse {
                            success: false,
                            session_id: None,
                            error_message: Some("Untrusted device: Pairing required".to_string()),
                        })
                    }
                }

                ControlMessage::PairRequest { client_id, client_name } => {
                    let challenge = self.pairing_manager.create_challenge(client_id.clone(), client_name.clone());
                    Some(ControlMessage::PairChallenge {
                        challenge_id: challenge.challenge_id,
                        pairing_code: challenge.code,
                        expires_in_secs: challenge.expires_in_secs as u32,
                    })
                }

                ControlMessage::PairVerify { challenge_id, pairing_code } => {
                    match self.pairing_manager.verify_code(&challenge_id, &pairing_code) {
                        Ok((client_id, token)) => {
                            self.config.trusted_tokens.write().insert(client_id.clone(), token.clone());
                            authenticated_client_id = Some(client_id.clone());
                            let session = self.session_manager.create_session(
                                client_id,
                                "Android Receiver".to_string(),
                                peer_addr,
                            );
                            active_session_id = Some(session.read().session_id);
                            Some(ControlMessage::PairResponse {
                                success: true,
                                auth_token: Some(token),
                                error_message: None,
                            })
                        }
                        Err(err) => Some(ControlMessage::PairResponse {
                            success: false,
                            auth_token: None,
                            error_message: Some(err.to_string()),
                        }),
                    }
                }

                ControlMessage::GetSources => {
                    let sources = vec![
                        AudioSourceInfo::new_system("src_sys", "Windows System Audio (Default Output)"),
                        AudioSourceInfo::new_process("src_chrome", "Google Chrome (Browser Audio)", 4892, "chrome.exe"),
                        AudioSourceInfo::new_process("src_spotify", "Spotify (Music Stream)", 8120, "spotify.exe"),
                        AudioSourceInfo::new_process("src_discord", "Discord (Voice & Communications)", 3216, "discord.exe"),
                    ];
                    Some(ControlMessage::SourceList { sources })
                }

                ControlMessage::SelectSource { source_id } => {
                    if let Some(sid) = active_session_id {
                        self.session_manager.set_session_source(sid, &source_id);
                        Some(ControlMessage::SourceChanged {
                            source_id: source_id.clone(),
                            fallback: false,
                            message: format!("Successfully switched to audio source: {}", source_id),
                        })
                    } else {
                        None
                    }
                }

                ControlMessage::SetProfile { profile } => {
                    if let Some(sid) = active_session_id {
                        self.session_manager.set_session_profile(sid, profile.clone());
                        Some(ControlMessage::ProfileChanged { profile })
                    } else {
                        None
                    }
                }

                ControlMessage::SetDeviceMute { is_muted, volume } => {
                    if let Some(sid) = active_session_id {
                        self.session_manager.set_session_mute(sid, is_muted, volume);
                        Some(ControlMessage::DeviceMuteChanged { is_muted, volume })
                    } else {
                        None
                    }
                }

                ControlMessage::GetSessions => {
                    let active = self.session_manager.list_active_sessions();
                    let values = active
                        .into_iter()
                        .map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null))
                        .collect();
                    Some(ControlMessage::SessionList { sessions: values })
                }

                ControlMessage::StartStream { media_port } => {
                    if let Some(sid) = active_session_id {
                        let media_addr = SocketAddr::new(peer_addr.ip(), media_port);
                        self.session_manager.set_streaming(
                            sid,
                            true,
                            Some(AudioProfile::default()),
                            Some(media_addr),
                        );
                        Some(ControlMessage::StreamStarted {
                            stream_id: 1,
                            session_id: sid,
                            assigned_port: self.config.media_port,
                            profile: AudioProfile::default(),
                        })
                    } else {
                        Some(ControlMessage::StreamStopped {
                            reason: "Unauthorized: must authenticate before starting stream".to_string(),
                        })
                    }
                }

                ControlMessage::StopStream => {
                    if let Some(sid) = active_session_id {
                        self.session_manager.set_streaming(sid, false, None, None);
                        Some(ControlMessage::StreamStopped {
                            reason: "Stopped by client".to_string(),
                        })
                    } else {
                        None
                    }
                }

                ControlMessage::RevokeDevice { client_id } => {
                    self.config.trusted_tokens.write().remove(&client_id);
                    self.session_manager.terminate_by_client_id(&client_id);
                    Some(ControlMessage::DeviceRevoked {
                        client_id,
                        success: true,
                    })
                }

                ControlMessage::Ping { client_send_time_us } => {
                    if let Some(sid) = active_session_id {
                        self.session_manager.record_heartbeat(sid);
                    }
                    Some(ControlMessage::Pong {
                        client_send_time_us,
                        server_recv_time_us: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as u64,
                    })
                }

                ControlMessage::Disconnect { reason: _ } => {
                    if let Some(sid) = active_session_id {
                        self.session_manager.terminate_session(sid);
                    }
                    break;
                }

                _ => None,
            };

            if let Some(resp) = response {
                let mut resp_str = serde_json::to_string(&resp)?;
                resp_str.push('\n');
                writer.write_all(resp_str.as_bytes()).await?;
                writer.flush().await?;
            }
        }

        if let Some(sid) = active_session_id {
            self.session_manager.terminate_session(sid);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn test_control_plane_server_pairing_and_streaming() {
        let trusted_tokens = Arc::new(RwLock::new(HashMap::new()));
        let config = ServerConfig {
            server_id: "test-pc".to_string(),
            server_name: "Desktop-WASAPI".to_string(),
            control_port: 0,
            media_port: 40121,
            trusted_tokens: trusted_tokens.clone(),
        };

        let pairing_mgr = Arc::new(PairingManager::new());
        let session_mgr = Arc::new(SessionManager::new());

        let server = Arc::new(ControlPlaneServer::new(config, pairing_mgr.clone(), session_mgr.clone()));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        // Spawn server loop
        let srv = server.clone();
        tokio::spawn(async move {
            let (stream, peer) = listener.accept().await.unwrap();
            srv.handle_client(stream, peer).await.unwrap();
        });

        // Connect client
        let stream = TcpStream::connect(local_addr).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        // Step 1: Send Hello
        let hello = ControlMessage::Hello {
            client_id: "test-phone".to_string(),
            client_name: "Pixel 9".to_string(),
            client_version: "1.0".to_string(),
            protocol_version: PROTOCOL_VERSION,
        };
        let mut hello_line = serde_json::to_string(&hello).unwrap();
        hello_line.push('\n');
        writer.write_all(hello_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        let ack_line = lines.next_line().await.unwrap().unwrap();
        let ack: ControlMessage = serde_json::from_str(&ack_line).unwrap();
        match ack {
            ControlMessage::HelloAck { requires_pairing, .. } => assert!(requires_pairing),
            _ => panic!("Expected HelloAck"),
        }

        // Step 2: Request Pairing
        let pair_req = ControlMessage::PairRequest {
            client_id: "test-phone".to_string(),
            client_name: "Pixel 9".to_string(),
        };
        let mut req_line = serde_json::to_string(&pair_req).unwrap();
        req_line.push('\n');
        writer.write_all(req_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        let chal_line = lines.next_line().await.unwrap().unwrap();
        let chal: ControlMessage = serde_json::from_str(&chal_line).unwrap();
        let (chal_id, code) = match chal {
            ControlMessage::PairChallenge { challenge_id, pairing_code, .. } => (challenge_id, pairing_code),
            _ => panic!("Expected PairChallenge"),
        };

        // Step 3: Verify Pairing Code
        let verify_req = ControlMessage::PairVerify {
            challenge_id: chal_id,
            pairing_code: code,
        };
        let mut v_line = serde_json::to_string(&verify_req).unwrap();
        v_line.push('\n');
        writer.write_all(v_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        let resp_line = lines.next_line().await.unwrap().unwrap();
        let pair_resp: ControlMessage = serde_json::from_str(&resp_line).unwrap();
        let token = match pair_resp {
            ControlMessage::PairResponse { success, auth_token, .. } => {
                assert!(success);
                auth_token.unwrap()
            }
            _ => panic!("Expected PairResponse"),
        };

        assert!(!token.is_empty());
        assert!(trusted_tokens.read().contains_key("test-phone"));

        // Step 4: Start Stream
        let start_msg = ControlMessage::StartStream { media_port: 40121 };
        let mut s_line = serde_json::to_string(&start_msg).unwrap();
        s_line.push('\n');
        writer.write_all(s_line.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        let stream_started_line = lines.next_line().await.unwrap().unwrap();
        let started: ControlMessage = serde_json::from_str(&stream_started_line).unwrap();
        match started {
            ControlMessage::StreamStarted { stream_id, session_id, .. } => {
                assert_eq!(stream_id, 1);
                assert!(session_id > 0);
            }
            _ => panic!("Expected StreamStarted"),
        }
    }
}
