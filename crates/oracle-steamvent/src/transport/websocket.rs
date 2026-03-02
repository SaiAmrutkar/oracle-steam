// crates/oracle-steamvent/src/transport/websocket.rs
use super::super::protocol::{SteamMessage, ProtocolHandler};
use anyhow::Result;
use std::sync::Arc;

pub struct WebSocketTransport {
    protocol: Arc<ProtocolHandler>,
}

impl WebSocketTransport {
    pub fn new(protocol: Arc<ProtocolHandler>) -> Self {
        Self { protocol }
    }

    /// Connect via WebSocket (stub for future implementation)
    pub async fn connect(&self, url: &str) -> Result<()> {
        println!("[WebSocket] Connecting to: {}", url);
        println!("[WebSocket] WebSocket support coming soon");
        Ok(())
    }
}
