// crates/steamclient/src/pipe.rs
pub struct SteamPipe {
    id: i32,
}

impl SteamPipe {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

// crates/steamclient/src/client.rs
pub struct SteamClient {
    steam_id: u64,
}

impl SteamClient {
    pub fn new(steam_id: u64) -> Self {
        Self { steam_id }
    }

    pub fn steam_id(&self) -> u64 {
        self.steam_id
    }
}

// crates/steamclient/src/auth.rs
pub struct AuthTicket {
    pub handle: u32,
    pub data: Vec<u8>,
}

impl AuthTicket {
    pub fn generate(steam_id: u64, app_id: u32) -> Self {
        let mut data = Vec::new();
        data.extend_from_slice(&steam_id.to_le_bytes());
        data.extend_from_slice(&app_id.to_le_bytes());
        data.extend_from_slice(&rand::random::<[u8; 16]>());

        Self {
            handle: rand::random(),
            data,
        }
    }
}

// crates/steamclient/src/connection.rs
use std::net::SocketAddr;

pub struct SteamConnection {
    pub remote_addr: SocketAddr,
    pub connected: bool,
}

impl SteamConnection {
    pub fn new() -> Self {
        Self {
            remote_addr: "127.0.0.1:27015".parse().unwrap(),
            connected: false,
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) -> Result<(), String> {
        self.remote_addr = addr;
        self.connected = true;
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }
}

// crates/steamclient/src/callbacks.rs
use oracle_callbacks::{queue_callback, CallbackData};

pub fn trigger_user_stats_received(game_id: u64, steam_id: u64) {
    queue_callback(oracle_callbacks::types::UserStatsReceived_t {
        game_id,
        result: 1,
        steam_id,
    });
}

pub fn trigger_lobby_created(lobby_id: u64, result: i32) {
    queue_callback(oracle_callbacks::types::LobbyCreated_t { result, lobby_id });
}

pub fn trigger_achievement_stored(game_id: u64, achievement_name: String) {
    queue_callback(oracle_callbacks::types::UserAchievementStored_t {
        game_id,
        group_achievement: false,
        achievement_name,
        cur_progress: 0,
        max_progress: 0,
    });
}
