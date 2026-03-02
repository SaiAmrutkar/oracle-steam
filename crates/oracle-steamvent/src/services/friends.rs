// crates/oracle-steamvent/src/services/friends.rs
use super::super::protocol::messages::*;
use anyhow::Result;
use tokio::sync::mpsc;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FriendData {
    pub steam_id: u64,
    pub persona_name: String,
    pub persona_state: u32,
    pub avatar_hash: Vec<u8>,
    pub relationship: u32,
    pub game_id: u64,
}

pub struct FriendsService {
    message_tx: mpsc::UnboundedSender<SteamMessage>,
    friends_list: Arc<RwLock<HashMap<u64, FriendData>>>,
    pending_invites: Arc<RwLock<Vec<u64>>>,
}

impl FriendsService {
    pub fn new(message_tx: mpsc::UnboundedSender<SteamMessage>) -> Self {
        Self {
            message_tx,
            friends_list: Arc::new(RwLock::new(HashMap::new())),
            pending_invites: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Sync friends list from server
    pub async fn sync_friends_list(&self) -> Result<Vec<FriendData>> {
        println!("[Friends] Syncing friends list");

        let message = SteamMessage::request_friends_list();
        self.message_tx.send(message)?;

        // In real implementation, wait for response
        // For now, return cached list
        Ok(self.friends_list.read().values().cloned().collect())
    }

    /// Add friend by Steam ID
    pub async fn add_friend(&self, steam_id: u64) -> Result<()> {
        println!("[Friends] Adding friend: {}", steam_id);

        self.pending_invites.write().push(steam_id);

        // TODO: Send friend invite message
        Ok(())
    }

    /// Remove friend
    pub async fn remove_friend(&self, steam_id: u64) -> Result<()> {
        println!("[Friends] Removing friend: {}", steam_id);

        self.friends_list.write().remove(&steam_id);

        // TODO: Send remove friend message
        Ok(())
    }

    /// Send chat message
    pub async fn send_chat_message(&self, recipient: u64, message: String) -> Result<()> {
        println!("[Friends] Sending message to {}: {}", recipient, message);

        let chat_msg = SteamMessage::send_chat(0, 0, message);
        self.message_tx.send(chat_msg)?;

        Ok(())
    }

    /// Update persona state
    pub async fn set_persona_state(&self, state: u32, name: Option<String>) -> Result<()> {
        println!("[Friends] Setting persona state: {}", state);

        // TODO: Send persona state update
        Ok(())
    }

    /// Get friend info
    pub fn get_friend(&self, steam_id: u64) -> Option<FriendData> {
        self.friends_list.read().get(&steam_id).cloned()
    }

    /// Get all friends
    pub fn get_all_friends(&self) -> Vec<FriendData> {
        self.friends_list.read().values().cloned().collect()
    }

    /// Update friend data (called when receiving persona state updates)
    pub fn update_friend(&self, friend_data: FriendData) {
        self.friends_list.write().insert(friend_data.steam_id, friend_data);
    }
}
