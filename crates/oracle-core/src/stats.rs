use crate::{AppId, StatValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub app_id: AppId,
    pub stats: HashMap<String, StatValue>,
    pub achievements: Vec<crate::PlayerAchievement>,
}

impl PlayerStats {
    pub fn new(app_id: AppId) -> Self {
        Self {
            app_id,
            stats: HashMap::new(),
            achievements: Vec::new(),
        }
    }

    pub fn set_stat(&mut self, name: String, value: StatValue) {
        self.stats.insert(name, value);
    }

    pub fn get_stat(&self, name: &str) -> Option<&StatValue> {
        self.stats.get(name)
    }

    pub fn unlock_achievement(&mut self, achievement_id: String) -> bool {
        if let Some(ach) = self
            .achievements
            .iter_mut()
            .find(|a| a.achievement_id == achievement_id)
        {
            if !ach.unlocked {
                ach.unlocked = true;
                ach.unlock_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return true;
            }
            return false;
        }

        self.achievements
            .push(crate::PlayerAchievement::new(achievement_id));
        true
    }
}
