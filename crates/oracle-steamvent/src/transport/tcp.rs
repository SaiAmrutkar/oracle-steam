// crates/oracle-steamvent/src/transport/tcp.rs
use super::super::protocol::{SteamMessage, ProtocolHandler};
use anyhow::Result;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;

pub struct TcpTransport {
    protocol: Arc<ProtocolHandler>,
}

impl TcpTransport {
    pub fn new(protocol: Arc<ProtocolHandler>) -> Self {
        Self { protocol }
    }

    /// Connect to Steam server via TCP
    pub async fn connect(&self, address: &str) -> Result<TcpStream> {
        println!("[TCP] Connecting to: {}", address);

        let stream = TcpStream::connect(address).await?;
        println!("[TCP] Connected successfully");

        Ok(stream)
    }

    /// Send message over TCP
    pub async fn send(&self, stream: &mut TcpStream, message: &SteamMessage) -> Result<()> {
        // Serialize message
        let data = self.protocol.serialize(message)?;

        // Write length prefix (4 bytes)
        stream.write_u32(data.len() as u32).await?;

        // Write data
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Receive message from TCP
    pub async fn receive(&self, stream: &mut TcpStream) -> Result<SteamMessage> {
        // Read length prefix
        let length = stream.read_u32().await? as usize;

        // Read data
        let mut buffer = vec![0u8; length];
        stream.read_exact(&mut buffer).await?;

        // Deserialize message
        let message = self.protocol.deserialize(&buffer)?;

        Ok(message)
    }

    /// Start message loop
    pub async fn message_loop<F>(&self, mut stream: TcpStream, mut handler: F) -> Result<()>
    where
        F: FnMut(SteamMessage) -> Result<()>,
    {
        loop {
            match self.receive(&mut stream).await {
                Ok(message) => {
                    if let Err(e) = handler(message) {
                        eprintln!("[TCP] Handler error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("[TCP] Receive error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
