// crates/oracle-steamvent/src/protocol/mod.rs
pub mod encryption;
pub mod messages;
pub mod serialization;

pub use encryption::ProtocolEncryption;
pub use messages::{SteamMessage, EMsgType};
pub use serialization::ProtocolSerializer;

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;

/// Protocol handler for Steam messages
pub struct ProtocolHandler {
    encryption: Arc<RwLock<ProtocolEncryption>>,
    serializer: Arc<ProtocolSerializer>,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            encryption: Arc::new(RwLock::new(ProtocolEncryption::new())),
            serializer: Arc::new(ProtocolSerializer::new()),
        }
    }

    /// Handle incoming Steam message
    pub fn handle_message(&self, message: SteamMessage) -> Result<()> {
        match message.msg_type {
            EMsgType::ClientLogonResponse => {
                println!("[Protocol] Logon response received");
            }
            EMsgType::ClientFriendsList => {
                println!("[Protocol] Friends list received");
            }
            EMsgType::ClientPersonaState => {
                println!("[Protocol] Persona state update");
            }
            _ => {
                println!("[Protocol] Message: {:?}", message.msg_type);
            }
        }
        Ok(())
    }

    /// Encrypt message payload
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.encryption.read().encrypt(data)
    }

    /// Decrypt message payload
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.encryption.read().decrypt(data)
    }

    /// Serialize message
    pub fn serialize(&self, message: &SteamMessage) -> Result<Vec<u8>> {
        self.serializer.serialize(message)
    }

    /// Deserialize message
    pub fn deserialize(&self, data: &[u8]) -> Result<SteamMessage> {
        self.serializer.deserialize(data)
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}
