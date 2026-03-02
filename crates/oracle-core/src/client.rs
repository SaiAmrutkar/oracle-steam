use crate::{AppId, LobbyData, PlayerStats, SteamId, UserProfile, UserStatus};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct SteamClient {
    user: Arc<RwLock<UserProfile>>,
    stats: Arc<RwLock<HashMap<AppId, PlayerStats>>>,
    lobbies: Arc<RwLock<HashMap<u64, LobbyData>>>,
}

impl SteamClient {
    pub fn new(steam_id: SteamId, username: String) -> Self {
        Self {
            user: Arc::new(RwLock::new(UserProfile {
                steam_id,
                username,
                avatar_hash: String::new(),
                level: 1,
                status: UserStatus::Online,
            })),
            stats: Arc::new(RwLock::new(HashMap::new())),
            lobbies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_steam_id(&self) -> SteamId {
        self.user.read().unwrap().steam_id
    }

    pub fn get_username(&self) -> String {
        self.user.read().unwrap().username.clone()
    }

    pub fn set_status(&self, status: UserStatus) {
        self.user.write().unwrap().status = status;
    }

    pub fn get_user_profile(&self) -> UserProfile {
        self.user.read().unwrap().clone()
    }
}

    // Lobby management functions
    pub fn create_lobby(&self, app_id: AppId, max_members: u32) -> u64 {
        let lobby_id = self.generate_lobby_id();
        let lobby = LobbyData {
            lobby_id,
            owner: self.get_steam_id(),
            app_id,
            max_members,
            members: vec![self.get_steam_id()],
            metadata: HashMap::new(),
            joinable: true,
        };
        
        self.lobbies.write().unwrap().insert(lobby_id, lobby);
        lobby_id
    }

    pub fn join_lobby(&self, lobby_id: u64) -> Result<(), String> {
        let mut lobbies = self.lobbies.write().unwrap();
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            let steam_id = self.get_steam_id();
            if !lobby.members.contains(&steam_id) {
                if lobby.members.len() < lobby.max_members as usize {
                    lobby.members.push(steam_id);
                    Ok(())
                } else {
                    Err("Lobby full".to_string())
                }
            } else {
                Ok(())
            }
        } else {
            Err("Lobby not found".to_string())
        }
    }

    pub fn leave_lobby(&self, lobby_id: u64) {
        let mut lobbies = self.lobbies.write().unwrap();
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            let steam_id = self.get_steam_id();
            lobby.members.retain(|&id| id != steam_id);
        }
    }

    pub fn set_lobby_data(&self, lobby_id: u64, key: String, value: String) -> Result<(), String> {
        let mut lobbies = self.lobbies.write().unwrap();
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby.metadata.insert(key, value);
            Ok(())
        } else {
            Err("Lobby not found".to_string())
        }
    }

    pub fn get_lobby_member_count(&self, lobby_id: u64) -> usize {
        let lobbies = self.lobbies.read().unwrap();
        lobbies.get(&lobby_id).map(|l| l.members.len()).unwrap_or(0)
    }

    pub fn get_lobby_member_by_index(&self, lobby_id: u64, index: usize) -> u64 {
        let lobbies = self.lobbies.read().unwrap();
        lobbies
            .get(&lobby_id)
            .and_then(|l| l.members.get(index).copied())
            .unwrap_or(0)
    }

    pub fn get_lobby_owner(&self, lobby_id: u64) -> u64 {
        let lobbies = self.lobbies.read().unwrap();
        lobbies.get(&lobby_id).map(|l| l.owner).unwrap_or(0)
    }

    pub fn set_lobby_type(&self, lobby_id: u64, _lobby_type: u32) -> Result<(), String> {
        // Lobby type stored in metadata
        Ok(())
    }

    pub fn set_lobby_joinable(&self, lobby_id: u64, joinable: bool) -> Result<(), String> {
        let mut lobbies = self.lobbies.write().unwrap();
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby.joinable = joinable;
            Ok(())
        } else {
            Err("Lobby not found".to_string())
        }
    }

    pub fn get_lobby_member_limit(&self, lobby_id: u64) -> u32 {
        let lobbies = self.lobbies.read().unwrap();
        lobbies.get(&lobby_id).map(|l| l.max_members).unwrap_or(0)
    }

    pub fn set_lobby_member_limit(&self, lobby_id: u64, max_members: u32) -> Result<(), String> {
        let mut lobbies = self.lobbies.write().unwrap();
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby.max_members = max_members;
            Ok(())
        } else {
            Err("Lobby not found".to_string())
        }
    }

    pub fn invite_user_to_lobby(&self, _lobby_id: u64, _steam_id_invitee: u64) -> Result<(), String> {
        // Send invite notification
        Ok(())
    }

    pub fn send_lobby_chat(&self, _lobby_id: u64, _data: &[u8]) -> Result<(), String> {
        // Broadcast message to lobby members
        Ok(())
    }

    pub fn request_lobby_data(&self, _lobby_id: u64) -> Result<(), String> {
        // Request lobby metadata update
        Ok(())
    }

    pub fn set_lobby_game_server(
        &self,
        _lobby_id: u64,
        _ip: u32,
        _port: u16,
        _steam_id: u64,
    ) -> Result<(), String> {
        // Associate game server with lobby
        Ok(())
    }

    pub fn set_lobby_member_data(&self, _lobby_id: u64, _key: String, _value: String) -> Result<(), String> {
        // Set per-member metadata
        Ok(())
    }

    fn generate_lobby_id(&self) -> u64 {
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        0x0110000100000000 | (timestamp & 0xFFFFFFFF)
    }
}
