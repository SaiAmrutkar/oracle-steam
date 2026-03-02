// crates/oracle-steamvent/src/services/content.rs
use super::super::protocol::messages::*;
use anyhow::{Result, bail};
use tokio::sync::mpsc;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use std::path::PathBuf;

pub struct ContentService {
    message_tx: mpsc::UnboundedSender<SteamMessage>,
    download_cache: Arc<RwLock<HashMap<u64, Vec<u8>>>>,
    ugc_metadata: Arc<RwLock<HashMap<u64, UGCDetails>>>,
    storage_path: PathBuf,
}

impl ContentService {
    pub fn new(message_tx: mpsc::UnboundedSender<SteamMessage>) -> Self {
        let storage_path = std::env::current_dir()
            .unwrap_or_default()
            .join("oracle_data")
            .join("content");

        std::fs::create_dir_all(&storage_path).ok();

        Self {
            message_tx,
            download_cache: Arc::new(RwLock::new(HashMap::new())),
            ugc_metadata: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        }
    }

    /// Download UGC content
    pub async fn download_content(&self, file_id: u64) -> Result<Vec<u8>> {
        println!("[Content] Downloading file: {}", file_id);

        // Check cache first
        if let Some(data) = self.download_cache.read().get(&file_id) {
            println!("[Content] Serving from cache");
            return Ok(data.clone());
        }

        // Request download from server
        // TODO: Send actual download request message

        // For now, check local storage
        let file_path = self.storage_path.join(format!("{}.dat", file_id));
        if file_path.exists() {
            let data = std::fs::read(&file_path)?;
            self.download_cache.write().insert(file_id, data.clone());
            return Ok(data);
        }

        bail!("Content not found: {}", file_id)
    }

    /// Upload content
    pub async fn upload_content(&self, data: Vec<u8>, filename: String) -> Result<u64> {
        println!("[Content] Uploading: {} ({} bytes)", filename, data.len());

        // Generate file ID
        let file_id = self.generate_file_id(&filename);

        // Save to local storage
        let file_path = self.storage_path.join(format!("{}.dat", file_id));
        std::fs::write(&file_path, &data)?;

        // Cache it
        self.download_cache.write().insert(file_id, data);

        // TODO: Send upload message to server

        Ok(file_id)
    }

    /// Get UGC details
    pub async fn get_ugc_details(&self, file_id: u64) -> Result<UGCDetails> {
        if let Some(details) = self.ugc_metadata.read().get(&file_id) {
            return Ok(details.clone());
        }

        // TODO: Request from server
        bail!("UGC details not found: {}", file_id)
    }

    /// Subscribe to workshop item
    pub async fn subscribe_item(&self, file_id: u64) -> Result<()> {
        println!("[Content] Subscribing to: {}", file_id);

        // TODO: Send subscribe message
        Ok(())
    }

    /// Unsubscribe from workshop item
    pub async fn unsubscribe_item(&self, file_id: u64) -> Result<()> {
        println!("[Content] Unsubscribing from: {}", file_id);

        // TODO: Send unsubscribe message
        Ok(())
    }

    /// List subscribed items
    pub fn list_subscribed(&self) -> Vec<u64> {
        self.ugc_metadata.read().keys().copied().collect()
    }

    fn generate_file_id(&self, filename: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        filename.hash(&mut hasher);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (hasher.finish() ^ timestamp) & 0x00FFFFFFFFFFFFFF
    }
}
