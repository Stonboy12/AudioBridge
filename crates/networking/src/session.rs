use audio_types::AudioProfile;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: u32,
    pub client_id: String,
    pub client_name: String,
    pub client_addr: String,
    pub media_port: u16,
    pub is_streaming: bool,
    pub active_source_id: String,
    pub active_source_name: String,
    pub profile_bitrate_kbps: u32,
    pub codec: String,
    pub is_muted: bool,
    pub volume: f32,
    pub duration_secs: u64,
}

pub struct ActiveSession {
    pub session_id: u32,
    pub client_id: String,
    pub client_name: String,
    pub client_addr: SocketAddr,
    pub media_addr: Option<SocketAddr>,
    pub is_streaming: bool,
    pub active_source_id: String,
    pub is_muted: bool,
    pub volume: f32,
    pub profile: Option<AudioProfile>,
    pub created_at: Instant,
    pub last_heartbeat: Instant,
}

pub struct SessionManager {
    next_session_id: AtomicU32,
    sessions: RwLock<HashMap<u32, Arc<RwLock<ActiveSession>>>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            next_session_id: AtomicU32::new(1001),
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Creates and registers a new isolated device session
    pub fn create_session(
        &self,
        client_id: String,
        client_name: String,
        client_addr: SocketAddr,
    ) -> Arc<RwLock<ActiveSession>> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let session = Arc::new(RwLock::new(ActiveSession {
            session_id,
            client_id,
            client_name,
            client_addr,
            media_addr: None,
            is_streaming: false,
            active_source_id: "src_sys".to_string(),
            is_muted: false,
            volume: 1.0,
            profile: Some(AudioProfile::default()),
            created_at: Instant::now(),
            last_heartbeat: Instant::now(),
        }));

        self.sessions.write().insert(session_id, session.clone());
        session
    }

    /// Retrieves an active session by ID
    pub fn get_session(&self, session_id: u32) -> Option<Arc<RwLock<ActiveSession>>> {
        self.sessions.read().get(&session_id).cloned()
    }

    /// Finds an active session by client ID
    pub fn find_by_client_id(&self, client_id: &str) -> Option<Arc<RwLock<ActiveSession>>> {
        let sessions = self.sessions.read();
        for session in sessions.values() {
            if session.read().client_id == client_id {
                return Some(session.clone());
            }
        }
        None
    }

    /// Updates the heartbeat timestamp for an active session
    pub fn record_heartbeat(&self, session_id: u32) -> bool {
        if let Some(session) = self.get_session(session_id) {
            session.write().last_heartbeat = Instant::now();
            true
        } else {
            false
        }
    }

    /// Sets streaming state and negotiates profile for an isolated session
    pub fn set_streaming(&self, session_id: u32, is_streaming: bool, profile: Option<AudioProfile>, media_addr: Option<SocketAddr>) -> bool {
        if let Some(session) = self.get_session(session_id) {
            let mut s = session.write();
            s.is_streaming = is_streaming;
            if let Some(p) = profile {
                s.profile = Some(p);
            }
            if let Some(addr) = media_addr {
                s.media_addr = Some(addr);
            }
            true
        } else {
            false
        }
    }

    /// Updates the active audio source for a specific device session
    pub fn set_session_source(&self, session_id: u32, source_id: &str) -> bool {
        if let Some(session) = self.get_session(session_id) {
            session.write().active_source_id = source_id.to_string();
            true
        } else {
            false
        }
    }

    /// Updates the audio profile configuration for a specific device session
    pub fn set_session_profile(&self, session_id: u32, profile: AudioProfile) -> bool {
        if let Some(session) = self.get_session(session_id) {
            session.write().profile = Some(profile);
            true
        } else {
            false
        }
    }

    /// Updates the mute state and volume multiplier for a specific device session
    pub fn set_session_mute(&self, session_id: u32, is_muted: bool, volume: f32) -> bool {
        if let Some(session) = self.get_session(session_id) {
            let mut s = session.write();
            s.is_muted = is_muted;
            s.volume = volume.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    /// Falls back all sessions using a closed/lost source to a fallback source (e.g. system audio)
    pub fn fallback_sessions_with_source(&self, old_source_id: &str, fallback_source_id: &str) -> Vec<u32> {
        let mut affected = Vec::new();
        let sessions = self.sessions.read();
        for (sid, session) in sessions.iter() {
            let mut s = session.write();
            if s.active_source_id == old_source_id {
                s.active_source_id = fallback_source_id.to_string();
                affected.push(*sid);
            }
        }
        affected
    }

    /// Returns a map of source_id -> count of active listeners
    pub fn get_active_source_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        let sessions = self.sessions.read();
        for session in sessions.values() {
            let s = session.read();
            if s.is_streaming {
                *counts.entry(s.active_source_id.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Returns all actively streaming sessions listening to a specific audio source
    pub fn get_streaming_sessions_for_source(&self, source_id: &str) -> Vec<Arc<RwLock<ActiveSession>>> {
        let sessions = self.sessions.read();
        sessions
            .values()
            .filter(|s| {
                let lock = s.read();
                lock.is_streaming && lock.active_source_id == source_id && lock.media_addr.is_some()
            })
            .cloned()
            .collect()
    }

    /// Returns all unique AudioProfiles needed for a source (enabling shared encoding fan-out)
    pub fn get_unique_profiles_for_source(&self, source_id: &str) -> Vec<AudioProfile> {
        let sessions = self.get_streaming_sessions_for_source(source_id);
        let mut unique = Vec::new();
        for s in sessions {
            let lock = s.read();
            if let Some(ref p) = lock.profile {
                if !unique.iter().any(|u: &AudioProfile| {
                    u.codec == p.codec && u.bitrate_kbps == p.bitrate_kbps && u.sample_rate == p.sample_rate && u.channels == p.channels
                }) {
                    unique.push(p.clone());
                }
            }
        }
        unique
    }

    /// Terminates a single session without disrupting any other active sessions
    pub fn terminate_session(&self, session_id: u32) -> bool {
        self.sessions.write().remove(&session_id).is_some()
    }

    /// Closes all sessions associated with a specific client (e.g. on revocation)
    pub fn terminate_by_client_id(&self, client_id: &str) -> usize {
        let mut sessions = self.sessions.write();
        let to_remove: Vec<u32> = sessions
            .iter()
            .filter(|(_, s)| s.read().client_id == client_id)
            .map(|(id, _)| *id)
            .collect();

        for id in &to_remove {
            sessions.remove(id);
        }
        to_remove.len()
    }

    /// Prunes stale sessions that have not reported a heartbeat within the timeout window
    pub fn cleanup_stale_sessions(&self, timeout: Duration) -> Vec<u32> {
        let mut sessions = self.sessions.write();
        let now = Instant::now();
        let mut pruned = Vec::new();

        sessions.retain(|id, session| {
            let elapsed = now.duration_since(session.read().last_heartbeat);
            if elapsed > timeout {
                pruned.push(*id);
                false
            } else {
                true
            }
        });

        pruned
    }

    /// Returns a list of all active session summaries for UI display
    pub fn list_active_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read();
        sessions
            .values()
            .map(|s| {
                let lock = s.read();
                let (bitrate, codec_name) = match lock.profile {
                    Some(ref p) => (p.bitrate_kbps, format!("{:?}", p.codec)),
                    None => (128, "Opus".to_string()),
                };

                let source_name = match lock.active_source_id.as_str() {
                    "src_sys" | "wasapi_default" => "System Audio (Default Speakers)".to_string(),
                    "src_chrome" => "Google Chrome".to_string(),
                    "src_spotify" => "Spotify".to_string(),
                    "src_discord" => "Discord".to_string(),
                    other => format!("Source ({})", other),
                };

                SessionInfo {
                    session_id: lock.session_id,
                    client_id: lock.client_id.clone(),
                    client_name: lock.client_name.clone(),
                    client_addr: lock.client_addr.to_string(),
                    media_port: lock.media_addr.map(|a| a.port()).unwrap_or(0),
                    is_streaming: lock.is_streaming,
                    active_source_id: lock.active_source_id.clone(),
                    active_source_name: source_name,
                    profile_bitrate_kbps: bitrate,
                    codec: codec_name,
                    is_muted: lock.is_muted,
                    volume: lock.volume,
                    duration_secs: lock.created_at.elapsed().as_secs(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_device_isolation_and_routing() {
        let manager = SessionManager::new();

        let s1 = manager.create_session("client-1".to_string(), "Phone A".to_string(), "192.168.1.50:40120".parse().unwrap());
        let s2 = manager.create_session("client-2".to_string(), "Tablet B".to_string(), "192.168.1.51:40120".parse().unwrap());
        let s3 = manager.create_session("client-3".to_string(), "Phone C".to_string(), "192.168.1.52:40120".parse().unwrap());
        let s4 = manager.create_session("client-4".to_string(), "Phone D".to_string(), "192.168.1.53:40120".parse().unwrap());

        let id1 = s1.read().session_id;
        let id2 = s2.read().session_id;
        let id3 = s3.read().session_id;
        let id4 = s4.read().session_id;

        // Device 1 and 2 listen to System Audio
        manager.set_session_source(id1, "src_sys");
        manager.set_streaming(id1, true, Some(AudioProfile::default()), Some("192.168.1.50:40121".parse().unwrap()));

        manager.set_session_source(id2, "src_sys");
        manager.set_streaming(id2, true, Some(AudioProfile::default()), Some("192.168.1.51:40121".parse().unwrap()));

        // Device 3 listens to Spotify
        manager.set_session_source(id3, "src_spotify");
        manager.set_streaming(id3, true, Some(AudioProfile::default()), Some("192.168.1.52:40121".parse().unwrap()));

        // Device 4 listens to Chrome with 64kbps Opus
        let mut low_prof = AudioProfile::default();
        low_prof.bitrate_kbps = 64;
        manager.set_session_source(id4, "src_chrome");
        manager.set_streaming(id4, true, Some(low_prof), Some("192.168.1.53:40121".parse().unwrap()));

        // Verify routing counts
        let counts = manager.get_active_source_counts();
        assert_eq!(counts.get("src_sys"), Some(&2));
        assert_eq!(counts.get("src_spotify"), Some(&1));
        assert_eq!(counts.get("src_chrome"), Some(&1));

        // Verify shared capture fanout: System Audio has 2 listeners with 1 unique profile
        let sys_listeners = manager.get_streaming_sessions_for_source("src_sys");
        assert_eq!(sys_listeners.len(), 2);
        let sys_profiles = manager.get_unique_profiles_for_source("src_sys");
        assert_eq!(sys_profiles.len(), 1, "Should only have 1 encoder instance for both System Audio listeners");

        // Test dynamic source switching on device 2 from System -> Spotify
        assert!(manager.set_session_source(id2, "src_spotify"));
        assert_eq!(manager.get_streaming_sessions_for_source("src_spotify").len(), 2);
        assert_eq!(manager.get_streaming_sessions_for_source("src_sys").len(), 1);

        // Test fallback: when Spotify process closes, fallback to src_sys
        let affected = manager.fallback_sessions_with_source("src_spotify", "src_sys");
        assert_eq!(affected.len(), 2);
        assert_eq!(manager.get_streaming_sessions_for_source("src_sys").len(), 3);
        assert_eq!(manager.get_streaming_sessions_for_source("src_spotify").len(), 0);

        // Disconnecting Device 1 leaves remaining devices streaming
        assert!(manager.terminate_session(id1));
        assert_eq!(manager.get_streaming_sessions_for_source("src_sys").len(), 2);
    }
}
