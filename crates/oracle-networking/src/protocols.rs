use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPacket {
    Handshake {
        steam_id: u64,
        username: String,
        version: u32,
    },
    HandshakeResponse {
        accepted: bool,
        server_time: u64,
    },
    P2PMessage {
        from: u64,
        to: u64,
        channel: u8,
        data: Vec<u8>,
        reliable: bool,
    },
    LobbyCreate {
        app_id: u32,
        max_members: u32,
        metadata: HashMap<String, String>,
    },
    LobbyCreated {
        lobby_id: u64,
    },
    LobbyJoin {
        lobby_id: u64,
    },
    LobbyJoined {
        lobby_id: u64,
        members: Vec<u64>,
    },
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameServer {
    pub server_id: u64,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub app_id: u32,
    pub players: u32,
    pub max_players: u32,
    pub map: String,
    pub region: String,
}
