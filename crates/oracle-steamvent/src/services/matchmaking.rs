// crates/oracle-steamvent/src/services/matchmaking.rs
use super::super::protocol::messages::*;
use anyhow::Result;
use tokio::sync::mpsc;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub address: String,
    pub port: u16,
    pub app_id: u32,
    pub region: String,
    pub players: u32,
    pub max_players: u32,
    pub ping: u32,
}

pub struct MatchmakingService {
    message_tx: mpsc::UnboundedSender<SteamMessage>,
    server_list: Arc<RwLock<Vec<ServerInfo>>>,
    lobbies: Arc<RwLock<HashMap<u64, LobbyCreateResponse>>>,
}

impl MatchmakingService {
    pub fn new(message_tx: mpsc::UnboundedSender<SteamMessage>) -> Self {
        Self {
            message_tx,
            server_list: Arc::new(RwLock::new(Vec::new())),
            lobbies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Request game server list
    pub async fn request_server_list(&self, app_id: u32, region: String) -> Result<Vec<String>> {
        println!("[Matchmaking] Requesting servers for app {} in {}", app_id, region);

        // TODO: Send server list request message

        // Return cached servers
        let servers = self.server_list.read();
        Ok(servers
            .iter()
            .filter(|s| s.app_id == app_id)
            .map(|s| format!("{}:{}", s.address, s.port))
            .collect())
    }

    /// Create lobby
    pub async fn create_lobby(&self, app_id: u32, max_members: u32) -> Result<u64> {
        println!("[Matchmaking] Creating lobby for app {}", app_id);

        // Generate lobby ID
        let lobby_id = self.generate_lobby_id();

        let response = LobbyCreateResponse {
            result: 1, // k_EResultOK
            lobby_id,
        };

        self.lobbies.write().insert(lobby_id, response);

        // TODO: Send lobby create message

        Ok(lobby_id)
    }

    /// Join lobby
    pub async fn join_lobby(&self, lobby_id: u64) -> Result<()> {
        println!("[Matchmaking] Joining lobby: {}", lobby_id);

        // TODO: Send lobby join message

        Ok(())
    }

    /// Leave lobby
    pub async fn leave_lobby(&self, lobby_id: u64) -> Result<()> {
        println!("[Matchmaking] Leaving lobby: {}", lobby_id);

        // TODO: Send lobby leave message

        Ok(())
    }

    /// Set lobby data
    pub async fn set_lobby_data(&self, lobby_id: u64, key: String, value: String) -> Result<()> {
        println!("[Matchmaking] Setting lobby data: {} = {}", key, value);

        // TODO: Send lobby data update message

        Ok(())
    }

    /// Get lobby count
    pub fn get_lobby_count(&self) -> usize {
        self.lobbies.read().len()
    }

    fn generate_lobby_id(&self) -> u64 {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate unique lobby ID
        0x0110000100000000 | (timestamp & 0xFFFFFFFF)
    }
}
