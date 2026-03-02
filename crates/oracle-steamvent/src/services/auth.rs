// crates/oracle-steamvent/src/services/auth.rs
use super::super::protocol::messages::*;
use anyhow::{Result, bail};
use tokio::sync::mpsc;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct AuthService {
    message_tx: mpsc::UnboundedSender<SteamMessage>,
    session_tickets: Arc<RwLock<HashMap<u64, Vec<u8>>>>,
    cached_credentials: Arc<RwLock<Option<(String, String)>>>,
}

impl AuthService {
    pub fn new(message_tx: mpsc::UnboundedSender<SteamMessage>) -> Self {
        Self {
            message_tx,
            session_tickets: Arc::new(RwLock::new(HashMap::new())),
            cached_credentials: Arc::new(RwLock::new(None)),
        }
    }

    /// Login with username and password
    pub async fn login(&self, username: String, password: String) -> Result<u64> {
        println!("[Auth] Logging in as: {}", username);

        // Cache credentials for reconnection
        *self.cached_credentials.write() = Some((username.clone(), password.clone()));

        // Send login message
        let message = SteamMessage::logon(username.clone(), password.clone());
        self.message_tx.send(message)?;

        // Generate Steam ID from username (in real impl, wait for server response)
        let steam_id = self.generate_steam_id(&username);

        println!("[Auth] Login successful. Steam ID: {}", steam_id);
        Ok(steam_id)
    }

    /// Login with auth ticket
    pub async fn login_with_ticket(&self, ticket: &[u8]) -> Result<u64> {
        println!("[Auth] Logging in with ticket ({} bytes)", ticket.len());

        // Decode ticket to get Steam ID
        let steam_id = self.decode_ticket(ticket)?;

        // Store ticket
        self.session_tickets.write().insert(steam_id, ticket.to_vec());

        Ok(steam_id)
    }

    /// Request encrypted app ticket
    pub async fn request_encrypted_ticket(&self, app_id: u32, user_data: Vec<u8>) -> Result<Vec<u8>> {
        println!("[Auth] Requesting encrypted ticket for app {}", app_id);

        // Generate ticket (in real impl, request from server)
        let ticket = self.generate_encrypted_ticket(app_id, &user_data)?;

        Ok(ticket)
    }

    /// Verify auth ticket
    pub fn verify_ticket(&self, ticket: &[u8]) -> Result<bool> {
        // Basic ticket validation
        if ticket.len() < 20 {
            return Ok(false);
        }

        // Check magic bytes
        if &ticket[0..4] != b"STKT" {
            return Ok(false);
        }

        Ok(true)
    }

    /// Generate Steam ID from username (deterministic)
    fn generate_steam_id(&self, username: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        username.hash(&mut hasher);
        let hash = hasher.finish();

        // Steam ID format: 76561197960265728 + hash % 1000000000
        76561197960265728 + (hash % 1000000000)
    }

    /// Decode ticket to extract Steam ID
    fn decode_ticket(&self, ticket: &[u8]) -> Result<u64> {
        if ticket.len() < 28 {
            bail!("Ticket too short");
        }

        // Skip magic bytes and read Steam ID
        let steam_id_bytes = &ticket[4..12];
        let steam_id = u64::from_le_bytes([
            steam_id_bytes[0], steam_id_bytes[1], steam_id_bytes[2], steam_id_bytes[3],
            steam_id_bytes[4], steam_id_bytes[5], steam_id_bytes[6], steam_id_bytes[7],
        ]);

        Ok(steam_id)
    }

    /// Generate encrypted app ticket (Oracle format)
    fn generate_encrypted_ticket(&self, app_id: u32, user_data: &[u8]) -> Result<Vec<u8>> {
        let mut ticket = Vec::new();

        // Magic: "ORAT" (Oracle Auth Ticket)
        ticket.extend_from_slice(b"ORAT");

        // Version: 1
        ticket.push(1);

        // App ID (4 bytes)
        ticket.extend_from_slice(&app_id.to_le_bytes());

        // Steam ID (8 bytes) - get from current session
        let steam_id = 76561198000000000u64; // TODO: Get from session
        ticket.extend_from_slice(&steam_id.to_le_bytes());

        // Timestamp (8 bytes)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        ticket.extend_from_slice(&timestamp.to_le_bytes());

        // User data length (2 bytes)
        ticket.extend_from_slice(&(user_data.len() as u16).to_le_bytes());

        // User data
        ticket.extend_from_slice(user_data);

        // Signature (32 bytes - SHA256 HMAC)
        let signature = self.sign_ticket(&ticket)?;
        ticket.extend_from_slice(&signature);

        Ok(ticket)
    }

    /// Sign ticket with HMAC-SHA256
    fn sign_ticket(&self, data: &[u8]) -> Result<[u8; 32]> {
        use sha2::{Sha256, Digest};

        // In production, use actual HMAC with secret key
        let mut hasher = Sha256::new();
        hasher.update(b"ORACLE_STEAM_SECRET_KEY");
        hasher.update(data);
        let result = hasher.finalize();

        let mut signature = [0u8; 32];
        signature.copy_from_slice(&result);

        Ok(signature)
    }

    /// Refresh authentication
    pub async fn refresh_auth(&self) -> Result<()> {
        if let Some((username, password)) = self.cached_credentials.read().clone() {
            println!("[Auth] Refreshing authentication");
            let message = SteamMessage::logon(username, password);
            self.message_tx.send(message)?;
        }
        Ok(())
    }

    /// Logout
    pub async fn logout(&self) -> Result<()> {
        println!("[Auth] Logging out");

        let message = SteamMessage::new(EMsgType::ClientLogOff, Vec::new());
        self.message_tx.send(message)?;

        *self.cached_credentials.write() = None;
        self.session_tickets.write().clear();

        Ok(())
    }
}
