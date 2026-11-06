use parking_lot::RwLock;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

pub const PAIRING_EXPIRY_SECS: u64 = 180; // 3 minutes validity per ADR-004
pub const MAX_PAIRING_ATTEMPTS: u32 = 3;  // Max 3 attempts before lockout per ADR-004

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PairingError {
    #[error("Pairing challenge not found or already completed")]
    NotFound,
    #[error("Pairing code has expired")]
    Expired,
    #[error("Incorrect pairing code. Remaining attempts: {0}")]
    IncorrectCode(u32),
    #[error("Maximum pairing attempts exceeded. Request a new pairing code")]
    MaxAttemptsExceeded,
    #[error("Pairing request was rejected by user")]
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingChallengeInfo {
    pub challenge_id: String,
    pub client_id: String,
    pub client_name: String,
    pub code: String,
    pub expires_in_secs: u64,
    pub attempts_remaining: u32,
}

struct InternalChallenge {
    challenge_id: String,
    client_id: String,
    client_name: String,
    code: String,
    created_at: Instant,
    attempts: u32,
}

pub struct PairingManager {
    challenges: RwLock<HashMap<String, InternalChallenge>>,
}

impl Default for PairingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PairingManager {
    pub fn new() -> Self {
        Self {
            challenges: RwLock::new(HashMap::new()),
        }
    }

    /// Creates a new pairing challenge with a 6-digit one-time PIN
    pub fn create_challenge(&self, client_id: String, client_name: String) -> PairingChallengeInfo {
        let mut rng = rand::thread_rng();
        let code_num: u32 = rng.gen_range(100_000..=999_999);
        let code = format!("{:03} {:03}", code_num / 1000, code_num % 1000);
        let challenge_id = format!("ch_{}_{:08x}", client_id, rng.gen::<u32>());

        let info = PairingChallengeInfo {
            challenge_id: challenge_id.clone(),
            client_id: client_id.clone(),
            client_name: client_name.clone(),
            code: code.clone(),
            expires_in_secs: PAIRING_EXPIRY_SECS,
            attempts_remaining: MAX_PAIRING_ATTEMPTS,
        };

        let internal = InternalChallenge {
            challenge_id: challenge_id.clone(),
            client_id,
            client_name,
            code,
            created_at: Instant::now(),
            attempts: 0,
        };

        self.challenges.write().insert(challenge_id, internal);
        info
    }

    /// Verifies the entered 6-digit PIN. On success, generates a 32-byte secure auth token
    pub fn verify_code(&self, challenge_id: &str, entered_code: &str) -> Result<(String, String), PairingError> {
        let mut challenges = self.challenges.write();
        let challenge = challenges.get_mut(challenge_id).ok_or(PairingError::NotFound)?;

        if challenge.created_at.elapsed() > Duration::from_secs(PAIRING_EXPIRY_SECS) {
            challenges.remove(challenge_id);
            return Err(PairingError::Expired);
        }

        if challenge.attempts >= MAX_PAIRING_ATTEMPTS {
            challenges.remove(challenge_id);
            return Err(PairingError::MaxAttemptsExceeded);
        }

        let clean_entered = entered_code.replace(' ', "");
        let clean_expected = challenge.code.replace(' ', "");

        if clean_entered == clean_expected {
            let client_id = challenge.client_id.clone();
            challenges.remove(challenge_id);
            let auth_token = generate_secure_auth_token();
            Ok((client_id, auth_token))
        } else {
            challenge.attempts += 1;
            let remaining = MAX_PAIRING_ATTEMPTS.saturating_sub(challenge.attempts);
            if remaining == 0 {
                challenges.remove(challenge_id);
                Err(PairingError::MaxAttemptsExceeded)
            } else {
                Err(PairingError::IncorrectCode(remaining))
            }
        }
    }

    /// Allows the desktop user to approve the challenge directly from the UI prompt
    pub fn approve_challenge(&self, challenge_id: &str) -> Result<(String, String), PairingError> {
        let mut challenges = self.challenges.write();
        let challenge = challenges.remove(challenge_id).ok_or(PairingError::NotFound)?;

        if challenge.created_at.elapsed() > Duration::from_secs(PAIRING_EXPIRY_SECS) {
            return Err(PairingError::Expired);
        }

        let client_id = challenge.client_id;
        let auth_token = generate_secure_auth_token();
        Ok((client_id, auth_token))
    }

    /// Rejects the pairing request
    pub fn reject_challenge(&self, challenge_id: &str) -> bool {
        self.challenges.write().remove(challenge_id).is_some()
    }

    /// Retrieves active pending challenges for the desktop UI
    pub fn list_pending_challenges(&self) -> Vec<PairingChallengeInfo> {
        let challenges = self.challenges.read();
        let now = Instant::now();

        challenges
            .values()
            .filter(|c| c.created_at.elapsed() <= Duration::from_secs(PAIRING_EXPIRY_SECS))
            .map(|c| {
                let elapsed = c.created_at.elapsed().as_secs();
                let remaining_secs = PAIRING_EXPIRY_SECS.saturating_sub(elapsed);
                let attempts_remaining = MAX_PAIRING_ATTEMPTS.saturating_sub(c.attempts);
                PairingChallengeInfo {
                    challenge_id: c.challenge_id.clone(),
                    client_id: c.client_id.clone(),
                    client_name: c.client_name.clone(),
                    code: c.code.clone(),
                    expires_in_secs: remaining_secs,
                    attempts_remaining,
                }
            })
            .collect()
    }
}

/// Generates a cryptographically strong 32-byte hex authentication token
pub fn generate_secure_auth_token() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_workflow_success() {
        let manager = PairingManager::new();
        let challenge = manager.create_challenge("android-1".to_string(), "Pixel 9".to_string());

        assert_eq!(challenge.code.len(), 7); // "XXX XXX"
        assert_eq!(challenge.attempts_remaining, 3);

        let result = manager.verify_code(&challenge.challenge_id, &challenge.code);
        assert!(result.is_ok());
        let (client_id, token) = result.unwrap();
        assert_eq!(client_id, "android-1");
        assert_eq!(token.len(), 64); // 32 bytes hex = 64 characters
    }

    #[test]
    fn test_pairing_incorrect_code_and_lockout() {
        let manager = PairingManager::new();
        let challenge = manager.create_challenge("android-2".to_string(), "Galaxy S24".to_string());

        // Attempt 1: wrong code
        let res1 = manager.verify_code(&challenge.challenge_id, "000 000");
        assert_eq!(res1, Err(PairingError::IncorrectCode(2)));

        // Attempt 2: wrong code
        let res2 = manager.verify_code(&challenge.challenge_id, "000 000");
        assert_eq!(res2, Err(PairingError::IncorrectCode(1)));

        // Attempt 3: wrong code -> lockout
        let res3 = manager.verify_code(&challenge.challenge_id, "000 000");
        assert_eq!(res3, Err(PairingError::MaxAttemptsExceeded));

        // Subsequent attempt returns NotFound because challenge was removed
        let res4 = manager.verify_code(&challenge.challenge_id, &challenge.code);
        assert_eq!(res4, Err(PairingError::NotFound));
    }

    #[test]
    fn test_desktop_direct_approval() {
        let manager = PairingManager::new();
        let challenge = manager.create_challenge("android-3".to_string(), "Tablet".to_string());

        let res = manager.approve_challenge(&challenge.challenge_id);
        assert!(res.is_ok());
        let (client_id, token) = res.unwrap();
        assert_eq!(client_id, "android-3");
        assert!(!token.is_empty());
    }
}
