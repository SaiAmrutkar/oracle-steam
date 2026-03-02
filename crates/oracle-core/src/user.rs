use crate::{AppId, SteamId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub steam_id: SteamId,
    pub username: String,
    pub avatar_hash: String,
    pub level: u32,
    pub status: UserStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Offline,
    Online,
    Away,
    Busy,
    InGame(AppId),
}

impl UserProfile {
    pub fn new(steam_id: SteamId, username: String) -> Self {
        Self {
            steam_id,
            username,
            avatar_hash: String::new(),
            level: 1,
            status: UserStatus::Online,
        }
    }

    pub fn is_online(&self) -> bool {
        !matches!(self.status, UserStatus::Offline)
    }

    pub fn is_in_game(&self) -> bool {
        matches!(self.status, UserStatus::InGame(_))
    }
}
