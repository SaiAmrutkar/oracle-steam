use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: u64,
    pub owner: u64,
    pub app_id: u32,
    pub max_members: u32,
    pub members: Vec<u64>,
    pub metadata: HashMap<String, String>,
    pub joinable: bool,
}

pub struct LobbyManager {
    lobbies: Arc<RwLock<HashMap<u64, Lobby>>>,
    my_lobbies: Arc<RwLock<Vec<u64>>>,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self {
            lobbies: Arc::new(RwLock::new(HashMap::new())),
            my_lobbies: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn create_lobby(&self, owner: u64, app_id: u32, max_members: u32) -> u64 {
        let lobby_id = rand::random::<u64>();

        let lobby = Lobby {
            id: lobby_id,
            owner,
            app_id,
            max_members,
            members: vec![owner],
            metadata: HashMap::new(),
            joinable: true,
        };

        self.lobbies.write().await.insert(lobby_id, lobby);
        self.my_lobbies.write().await.push(lobby_id);

        println!("[Lobby] Created lobby {} for app {}", lobby_id, app_id);
        lobby_id
    }

    pub async fn join_lobby(&self, lobby_id: u64, member_id: u64) -> Result<(), String> {
        let mut lobbies = self.lobbies.write().await;

        let lobby = lobbies
            .get_mut(&lobby_id)
            .ok_or_else(|| "Lobby not found".to_string())?;

        if lobby.members.len() >= lobby.max_members as usize {
            return Err("Lobby is full".to_string());
        }

        if !lobby.joinable {
            return Err("Lobby is not joinable".to_string());
        }

        if !lobby.members.contains(&member_id) {
            lobby.members.push(member_id);
            println!("[Lobby] Player {} joined lobby {}", member_id, lobby_id);
        }

        Ok(())
    }

    pub async fn leave_lobby(&self, lobby_id: u64, member_id: u64) {
        let mut lobbies = self.lobbies.write().await;

        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby.members.retain(|&id| id != member_id);
            println!("[Lobby] Player {} left lobby {}", member_id, lobby_id);

            if lobby.members.is_empty() {
                lobbies.remove(&lobby_id);
                self.my_lobbies.write().await.retain(|&id| id != lobby_id);
                println!("[Lobby] Lobby {} disbanded (empty)", lobby_id);
            } else if lobby.owner == member_id && !lobby.members.is_empty() {
                lobby.owner = lobby.members[0];
                println!("[Lobby] Ownership transferred to {}", lobby.owner);
            }
        }
    }

    pub async fn set_lobby_data(
        &self,
        lobby_id: u64,
        key: String,
        value: String,
    ) -> Result<(), String> {
        let mut lobbies = self.lobbies.write().await;
        let lobby = lobbies
            .get_mut(&lobby_id)
            .ok_or_else(|| "Lobby not found".to_string())?;

        lobby.metadata.insert(key, value);
        Ok(())
    }

    pub async fn get_lobby_members(&self, lobby_id: u64) -> Vec<u64> {
        self.lobbies
            .read()
            .await
            .get(&lobby_id)
            .map(|l| l.members.clone())
            .unwrap_or_default()
    }
}
