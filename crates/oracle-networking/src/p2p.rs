use crate::NetworkPacket;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct P2PConnection {
    pub peer_id: u64,
    pub address: SocketAddr,
    pub last_seen: std::time::Instant,
    pub rtt: u64,
    pub packet_loss: f32,
}

pub struct P2PManager {
    connections: Arc<RwLock<HashMap<u64, P2PConnection>>>,
    packet_tx: mpsc::UnboundedSender<(u64, NetworkPacket)>,
    packet_rx: Arc<RwLock<mpsc::UnboundedReceiver<(u64, NetworkPacket)>>>,
}

impl P2PManager {
    pub fn new() -> Self {
        let (packet_tx, packet_rx) = mpsc::unbounded_channel();

        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            packet_tx,
            packet_rx: Arc::new(RwLock::new(packet_rx)),
        }
    }

    pub async fn connect_to_peer(&self, peer_id: u64, address: SocketAddr) {
        let connection = P2PConnection {
            peer_id,
            address,
            last_seen: std::time::Instant::now(),
            rtt: 0,
            packet_loss: 0.0,
        };

        self.connections.write().await.insert(peer_id, connection);
        println!("[P2P] Connected to peer: {} at {}", peer_id, address);
    }

    pub async fn disconnect_peer(&self, peer_id: u64) {
        self.connections.write().await.remove(&peer_id);
        println!("[P2P] Disconnected from peer: {}", peer_id);
    }

    pub async fn send_packet(
        &self,
        to: u64,
        packet: NetworkPacket,
        _reliable: bool,
    ) -> Result<(), String> {
        let connections = self.connections.read().await;

        if connections.contains_key(&to) {
            self.packet_tx
                .send((to, packet))
                .map_err(|e| format!("Failed to send packet: {}", e))?;
            Ok(())
        } else {
            Err(format!("No connection to peer {}", to))
        }
    }

    pub async fn get_peer_rtt(&self, peer_id: u64) -> Option<u64> {
        self.connections.read().await.get(&peer_id).map(|c| c.rtt)
    }

    pub async fn update_connection_stats(&self, peer_id: u64, rtt: u64, packet_loss: f32) {
        if let Some(conn) = self.connections.write().await.get_mut(&peer_id) {
            conn.rtt = rtt;
            conn.packet_loss = packet_loss;
            conn.last_seen = std::time::Instant::now();
        }
    }
}
