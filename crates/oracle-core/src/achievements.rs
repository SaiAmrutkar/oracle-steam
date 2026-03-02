use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub icon_gray: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAchievement {
    pub achievement_id: String,
    pub unlocked: bool,
    pub unlock_time: u64,
}

impl PlayerAchievement {
    pub fn new(achievement_id: String) -> Self {
        Self {
            achievement_id,
            unlocked: true,
            unlock_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }
}
