use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub steam_id: u64,
    pub session_token: String,
    pub web_auth_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct SessionManager {
    steam_id: u64,
    session_data: Arc<RwLock<Option<SessionData>>>,
    cookies: Arc<RwLock<HashMap<String, String>>>,
}

impl SessionManager {
    pub fn new(steam_id: u64) -> Result<Self> {
        Ok(Self {
            steam_id,
            session_data: Arc::new(RwLock::new(None)),
            cookies: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn update_from_login(
        &self,
        auth_result: &super::protocol::AuthenticationResult,
    ) -> Result<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let session = SessionData {
            steam_id: auth_result.steam_id,
            session_token: auth_result.session_token.clone(),
            web_auth_token: auth_result.session_token.clone(),
            access_token: self.generate_access_token(&auth_result.session_token)?,
            refresh_token: self.generate_refresh_token(&auth_result.session_token)?,
            created_at: current_time,
            expires_at: current_time + 86400, // 24 hours
        };

        *self.session_data.write() = Some(session);

        // Save session to disk
        self.save_session()?;

        Ok(())
    }

    pub fn get_web_auth_token(&self) -> Result<String> {
        let session = self.session_data.read();

        match session.as_ref() {
            Some(s) => {
                if self.is_session_valid(s) {
                    Ok(s.web_auth_token.clone())
                } else {
                    anyhow::bail!("Session expired")
                }
            }
            None => anyhow::bail!("No active session"),
        }
    }

    pub fn get_access_token(&self) -> Result<String> {
        let session = self.session_data.read();

        match session.as_ref() {
            Some(s) => {
                if self.is_session_valid(s) {
                    Ok(s.access_token.clone())
                } else {
                    anyhow::bail!("Session expired")
                }
            }
            None => anyhow::bail!("No active session"),
        }
    }

    pub fn refresh_session(&self) -> Result<()> {
        let mut session = self.session_data.write();

        if let Some(ref mut s) = *session {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();

            s.expires_at = current_time + 86400;
            s.access_token = self.generate_access_token(&s.session_token)?;
        }

        Ok(())
    }

    pub fn set_cookie(&self, name: String, value: String) {
        self.cookies.write().insert(name, value);
    }

    pub fn get_cookie(&self, name: &str) -> Option<String> {
        self.cookies.read().get(name).cloned()
    }

    pub fn get_all_cookies(&self) -> HashMap<String, String> {
        self.cookies.read().clone()
    }

    fn is_session_valid(&self, session: &SessionData) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        current_time < session.expires_at
    }

    fn generate_access_token(&self, session_token: &str) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(session_token.as_bytes());
        hasher.update(b"OracleSteamAccessToken");

        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    fn generate_refresh_token(&self, session_token: &str) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(session_token.as_bytes());
        hasher.update(b"OracleSteamRefreshToken");

        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    fn save_session(&self) -> Result<()> {
        let session = self.session_data.read();

        if let Some(ref s) = *session {
            let session_path = self.get_session_path()?;

            if let Some(parent) = session_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let json = serde_json::to_string_pretty(s)?;
            std::fs::write(session_path, json)?;
        }

        Ok(())
    }

    pub fn load_session(&self) -> Result<bool> {
        let session_path = self.get_session_path()?;

        if !session_path.exists() {
            return Ok(false);
        }

        let json = std::fs::read_to_string(session_path)?;
        let session: SessionData = serde_json::from_str(&json)?;

        if self.is_session_valid(&session) {
            *self.session_data.write() = Some(session);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_session_path(&self) -> Result<std::path::PathBuf> {
        let mut path = oracle_core::config::get_data_dir()?;
        path.push("sessions");
        path.push(format!("{}.json", self.steam_id));
        Ok(path)
    }
}
