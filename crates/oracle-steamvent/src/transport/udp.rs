// crates/oracle-steamvent/src/transport/udp.rs
use super::super::protocol::{SteamMessage, ProtocolHandler};
use anyhow::Result;
use tokio::net::UdpSocket;
use std::sync::Arc;
use std::net::SocketAddr;

pub struct UdpTransport {
    protocol: Arc<ProtocolHandler>,
}

impl UdpTransport {
    pub fn new(protocol: Arc<ProtocolHandler>) -> Self {
        Self { protocol }
    }

    /// Bind UDP socket
    pub async fn bind(&self, address: &str) -> Result<UdpSocket> {
        println!("[UDP] Binding to: {}", address);

        let socket = UdpSocket::bind(address).await?;
        println!("[UDP] Bound successfully");

        Ok(socket)
    }

    /// Send message via UDP
    pub async fn send_to(&self, socket: &UdpSocket, message: &SteamMessage, addr: SocketAddr) -> Result<()> {
        let data = self.protocol.serialize(message)?;
        socket.send_to(&data, addr).await?;
        Ok(())
    }

    /// Receive message from UDP
    pub async fn receive_from(&self, socket: &UdpSocket) -> Result<(SteamMessage, SocketAddr)> {
        let mut buffer = vec![0u8; 65536];
        let (len, addr) = socket.recv_from(&mut buffer).await?;

        buffer.truncate(len);
        let message = self.protocol.deserialize(&buffer)?;

        Ok((message, addr))
    }

    /// Start UDP message loop
    pub async fn message_loop<F>(&self, socket: UdpSocket, mut handler: F) -> Result<()>
    where
        F: FnMut(SteamMessage, SocketAddr) -> Result<()>,
    {
        loop {
            match self.receive_from(&socket).await {
                Ok((message, addr)) => {
                    if let Err(e) = handler(message, addr) {
                        eprintln!("[UDP] Handler error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("[UDP] Receive error: {}", e);
                }
            }
        }
    }
}
