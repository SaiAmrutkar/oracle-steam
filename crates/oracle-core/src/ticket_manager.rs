// crates/oracle-core/src/ticket_manager.rs
// Manages encrypted app tickets with intelligent caching to work around Denuvo limits

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Maximum tickets per app per day (Denuvo limit)
const MAX_TICKETS_PER_DAY: usize = 5;

/// How long a ticket remains valid (15 minutes for Denuvo)
const TICKET_VALIDITY_DURATION: Duration = Duration::from_secs(15 * 60);

/// How long before expiry to generate a new ticket (2 minutes buffer)
const TICKET_REFRESH_BUFFER: Duration = Duration::from_secs(2 * 60);

/// Manages encrypted app tickets with caching and rotation
pub struct TicketManager {
    /// Active tickets per app ID
    tickets: Arc<RwLock<HashMap<u32, TicketCache>>>,
    
    /// Storage path for persistent tickets
    storage_path: PathBuf,
    
    /// Steam CM client for generating new tickets
    cm_client: Arc<RwLock<Option<crate::steamvent::SteamCMClient>>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct TicketCache {
    app_id: u32,
    
    /// Current active ticket
    current: Option<CachedTicket>,
    
    /// Previously generated tickets (for rotation)
    history: Vec<CachedTicket>,
    
    /// Generation count today
    generated_today: usize,
    
    /// When the day counter resets
    reset_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedTicket {
    /// Base64-encoded ticket data
    ticket: String,
    
    /// When this ticket was generated
    generated_at: u64,
    
    /// When this ticket expires
    expires_at: u64,
    
    /// How many times this ticket has been used
    use_count: usize,
    
    /// SteamID this ticket is for
    steam_id: u64,
}

impl TicketManager {
    pub fn new(storage_path: PathBuf) -> Self {
        std::fs::create_dir_all(&storage_path).ok();
        
        Self {
            tickets: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
            cm_client: Arc::new(RwLock::new(None)),
        }
    }

    /// Set Steam CM client for ticket generation
    pub async fn set_cm_client(&self, client: crate::steamvent::SteamCMClient) {
        *self.cm_client.write().await = Some(client);
    }

    /// Get a valid encrypted app ticket for the given app
    pub async fn get_ticket(&self, app_id: u32, steam_id: u64) -> Result<String> {
        let mut tickets = self.tickets.write().await;
        
        let cache = tickets.entry(app_id).or_insert_with(|| TicketCache {
            app_id,
            current: None,
            history: Vec::new(),
            generated_today: 0,
            reset_time: Self::next_reset_time(),
        });

        // Check if we need to reset daily counter
        if Self::current_timestamp() >= cache.reset_time {
            cache.generated_today = 0;
            cache.reset_time = Self::next_reset_time();
        }

        // Check if current ticket is still valid
        if let Some(current) = &cache.current {
            let now = Self::current_timestamp();
            let time_remaining = current.expires_at.saturating_sub(now);
            
            // If ticket has enough time left, reuse it
            if time_remaining > TICKET_REFRESH_BUFFER.as_secs() {
                println!("[Ticket] Reusing cached ticket for app {} ({}s remaining)", 
                         app_id, time_remaining);
                return Ok(current.ticket.clone());
            }
        }

        // Try to find a usable ticket from history
        for cached in &cache.history {
            let now = Self::current_timestamp();
            if cached.expires_at > now + TICKET_REFRESH_BUFFER.as_secs() {
                println!("[Ticket] Rotating to historical ticket for app {}", app_id);
                cache.current = Some(cached.clone());
                self.save_cache(app_id, cache).await?;
                return Ok(cached.ticket.clone());
            }
        }

        // Need to generate a new ticket
        if cache.generated_today >= MAX_TICKETS_PER_DAY {
            // We've hit the daily limit - try to reuse the most recent ticket anyway
            if let Some(current) = &cache.current {
                println!("[Ticket] WARNING: Daily limit reached for app {}, reusing expired ticket", 
                         app_id);
                return Ok(current.ticket.clone());
            }
            
            bail!("Daily ticket limit reached ({}/{}) for app {}", 
                  cache.generated_today, MAX_TICKETS_PER_DAY, app_id);
        }

        // Generate new ticket from Steam
        let ticket_data = self.generate_ticket_from_steam(app_id, steam_id).await?;
        
        let new_ticket = CachedTicket {
            ticket: ticket_data,
            generated_at: Self::current_timestamp(),
            expires_at: Self::current_timestamp() + TICKET_VALIDITY_DURATION.as_secs(),
            use_count: 0,
            steam_id,
        };

        // Move current ticket to history
        if let Some(old) = cache.current.take() {
            cache.history.push(old);
            
            // Keep only last 10 tickets in history
            if cache.history.len() > 10 {
                cache.history.remove(0);
            }
        }

        cache.current = Some(new_ticket.clone());
        cache.generated_today += 1;

        self.save_cache(app_id, cache).await?;

        println!("[Ticket] Generated new ticket for app {} ({}/{})", 
                 app_id, cache.generated_today, MAX_TICKETS_PER_DAY);

        Ok(new_ticket.ticket)
    }

    /// Generate ticket directly from Steam servers
    async fn generate_ticket_from_steam(&self, app_id: u32, steam_id: u64) -> Result<String> {
        let client = self.cm_client.read().await;
        
        let cm_client = client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CM client not initialized"))?;

        // Request ticket from Steam
        let ticket_bytes = cm_client.request_encrypted_app_ticket(app_id).await?;

        // Encode as base64
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let encoded = STANDARD.encode(&ticket_bytes);

        Ok(encoded)
    }

    /// Load cached tickets from disk
    pub async fn load_from_disk(&self) -> Result<()> {
        let path = self.storage_path.join("tickets.json");
        
        if !path.exists() {
            return Ok(());
        }

        let data = tokio::fs::read_to_string(&path).await?;
        let loaded: HashMap<u32, TicketCache> = serde_json::from_str(&data)?;

        *self.tickets.write().await = loaded;

        println!("[Ticket] Loaded {} cached tickets from disk", 
                 self.tickets.read().await.len());

        Ok(())
    }

    async fn save_cache(&self, app_id: u32, cache: &TicketCache) -> Result<()> {
        // Save to memory
        // (Already done by the caller)

        // Save to disk
        let path = self.storage_path.join("tickets.json");
        let tickets = self.tickets.read().await;
        let data = serde_json::to_string_pretty(&*tickets)?;
        
        tokio::fs::write(&path, data).await?;

        Ok(())
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn next_reset_time() -> u64 {
        // Reset at midnight UTC
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let seconds_in_day = 24 * 60 * 60;
        let current_day_start = (now / seconds_in_day) * seconds_in_day;
        
        current_day_start + seconds_in_day
    }

    /// Manually add a ticket (useful for importing from real Steam client)
    pub async fn import_ticket(
        &self,
        app_id: u32,
        steam_id: u64,
        ticket_base64: String,
    ) -> Result<()> {
        let mut tickets = self.tickets.write().await;
        
        let cache = tickets.entry(app_id).or_insert_with(|| TicketCache {
            app_id,
            current: None,
            history: Vec::new(),
            generated_today: 0,
            reset_time: Self::next_reset_time(),
        });

        let imported = CachedTicket {
            ticket: ticket_base64,
            generated_at: Self::current_timestamp(),
            expires_at: Self::current_timestamp() + TICKET_VALIDITY_DURATION.as_secs(),
            use_count: 0,
            steam_id,
        };

        cache.current = Some(imported);

        self.save_cache(app_id, cache).await?;

        println!("[Ticket] Imported ticket for app {}", app_id);

        Ok(())
    }

    /// Get statistics about ticket usage
    pub async fn get_stats(&self, app_id: u32) -> Option<TicketStats> {
        let tickets = self.tickets.read().await;
        let cache = tickets.get(&app_id)?;

        Some(TicketStats {
            app_id,
            generated_today: cache.generated_today,
            remaining_today: MAX_TICKETS_PER_DAY.saturating_sub(cache.generated_today),
            has_active_ticket: cache.current.is_some(),
            history_count: cache.history.len(),
            reset_time: cache.reset_time,
        })
    }

    /// Prune expired tickets from history
    pub async fn prune_expired(&self) -> usize {
        let mut tickets = self.tickets.write().await;
        let mut pruned = 0;
        let now = Self::current_timestamp();

        for cache in tickets.values_mut() {
            let before = cache.history.len();
            cache.history.retain(|t| t.expires_at > now);
            pruned += before - cache.history.len();
        }

        println!("[Ticket] Pruned {} expired tickets", pruned);
        pruned
    }
}

#[derive(Debug, Clone)]
pub struct TicketStats {
    pub app_id: u32,
    pub generated_today: usize,
    pub remaining_today: usize,
    pub has_active_ticket: bool,
    pub history_count: usize,
    pub reset_time: u64,
}

/// Helper tool to extract tickets from real Steam client
pub async fn extract_ticket_from_steam(app_id: u32) -> Result<(u64, String)> {
    // This requires the actual Steam client to be running
    // We use the real Steam API to generate a ticket
    
    unsafe {
        std::env::set_var("SteamAppId", app_id.to_string());
        std::env::set_var("SteamGameId", app_id.to_string());

        // Initialize Steam API
        let init_result = steamworks_sys::SteamAPI_InitFlat(std::ptr::null_mut());
        if init_result != steamworks_sys::ESteamAPIInitResult::k_ESteamAPIInitResult_OK {
            bail!("Failed to initialize Steam API - is Steam running?");
        }

        let user = steamworks_sys::SteamAPI_SteamUser_v023();

        // Request encrypted ticket
        steamworks_sys::SteamAPI_ISteamUser_RequestEncryptedAppTicket(
            user,
            std::ptr::null_mut(),
            0
        );

        // Wait for callback
        let pipe = steamworks_sys::SteamAPI_GetHSteamPipe();
        
        let ticket = loop {
            steamworks_sys::SteamAPI_ManualDispatch_RunFrame(pipe);
            
            let mut callback: steamworks_sys::CallbackMsg_t = std::mem::zeroed();
            
            if steamworks_sys::SteamAPI_ManualDispatch_GetNextCallback(pipe, &mut callback) {
                if callback.m_iCallback == steamworks_sys::SteamAPICallCompleted_t_k_iCallback as i32 {
                    let apicall = &*(callback.m_pubParam as *const steamworks_sys::SteamAPICallCompleted_t);
                    
                    if apicall.m_iCallback == steamworks_sys::EncryptedAppTicketResponse_t_k_iCallback as i32 {
                        // Get ticket data
                        let mut ticket_data = vec![0u8; 2048];
                        let mut ticket_len = 0;
                        
                        let success = steamworks_sys::SteamAPI_ISteamUser_GetEncryptedAppTicket(
                            user,
                            ticket_data.as_mut_ptr() as *mut _,
                            2048,
                            &mut ticket_len
                        );

                        if success {
                            ticket_data.truncate(ticket_len as usize);
                            
                            use base64::{Engine as _, engine::general_purpose::STANDARD};
                            let encoded = STANDARD.encode(&ticket_data);
                            
                            let steam_id = steamworks_sys::SteamAPI_ISteamUser_GetSteamID(user);
                            
                            steamworks_sys::SteamAPI_ManualDispatch_FreeLastCallback(pipe);
                            steamworks_sys::SteamAPI_Shutdown();
                            
                            break Ok((steam_id, encoded));
                        }
                    }
                }
                
                steamworks_sys::SteamAPI_ManualDispatch_FreeLastCallback(pipe);
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        ticket
    }
}