// crates/oracle-steamvent/src/protocol/serialization.rs
use super::messages::{SteamMessage, EMsgType};
use anyhow::{Result, bail};

pub struct ProtocolSerializer;

impl ProtocolSerializer {
    pub fn new() -> Self {
        Self
    }

    /// Serialize Steam message to wire format
    pub fn serialize(&self, message: &SteamMessage) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        // Write message type (4 bytes)
        buffer.extend_from_slice(&(message.msg_type as u32).to_le_bytes());
        
        // Write payload length (4 bytes)
        buffer.extend_from_slice(&(message.payload.len() as u32).to_le_bytes());
        
        // Write payload
        buffer.extend_from_slice(&message.payload);
        
        Ok(buffer)
    }

    /// Deserialize Steam message from wire format
    pub fn deserialize(&self, data: &[u8]) -> Result<SteamMessage> {
        if data.len() < 8 {
            bail!("Message too short");
        }

        // Read message type
        let msg_type_raw = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let msg_type = match msg_type_raw {
            701 => EMsgType::ClientLogon,
            751 => EMsgType::ClientLogonResponse,
            702 => EMsgType::ClientLogOff,
            703 => EMsgType::ClientHeartBeat,
            767 => EMsgType::ClientFriendsList,
            766 => EMsgType::ClientPersonaState,
            797 => EMsgType::ClientChatMsg,
            5613 => EMsgType::ClientGetUserStats,
            5611 => EMsgType::ClientStoreUserStats,
            9535 => EMsgType::ClientLobbyCreate,
            9537 => EMsgType::ClientLobbyJoin,
            _ => EMsgType::Invalid,
        };

        // Read payload length
        let payload_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

        if data.len() < 8 + payload_len {
            bail!("Incomplete message payload");
        }

        // Read payload
        let payload = data[8..8 + payload_len].to_vec();

        Ok(SteamMessage::new(msg_type, payload))
    }
}

impl Default for ProtocolSerializer {
    fn default() -> Self {
        Self::new()
    }
}
