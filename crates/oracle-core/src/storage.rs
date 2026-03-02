use crate::{AppId, PlayerStats, Result, SteamEmuError, SteamId, UserProfile};
use std::path::PathBuf;

pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).ok();
        Self { base_path }
    }

    pub fn save_stats(&self, steam_id: SteamId, stats: &PlayerStats) -> Result<()> {
        let path = self
            .base_path
            .join(format!("stats_{}_{}.json", steam_id, stats.app_id));
        let json = serde_json::to_string_pretty(stats)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_stats(&self, steam_id: SteamId, app_id: AppId) -> Result<PlayerStats> {
        let path = self
            .base_path
            .join(format!("stats_{}_{}.json", steam_id, app_id));
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn save_user_profile(&self, profile: &UserProfile) -> Result<()> {
        let path = self
            .base_path
            .join(format!("user_{}.json", profile.steam_id));
        let json = serde_json::to_string_pretty(profile)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_user_profile(&self, steam_id: SteamId) -> Result<UserProfile> {
        let path = self.base_path.join(format!("user_{}.json", steam_id));
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}
