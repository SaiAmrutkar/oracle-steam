use super::EncryptedAppTicket;
use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes256;
use anyhow::{bail, Context, Result};
use parking_lot::RwLock;
use rand::Rng;
use sha1::{Digest, Sha1};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const TICKET_VERSION: u32 = 4;
const STEAM_ENCRYPTION_KEY: &[u8] = b"3e878c3e8f3e878c3e8f3e878c3e8f3e"; // Example key, real one is different

pub struct TicketGenerator {
    cache: Arc<super::TicketCache>,
    encryption_key: [u8; 32],
    use_real_steam: bool,
}

impl TicketGenerator {
    pub fn new(use_real_steam: bool) -> Result<Arc<Self>> {
        let cache = super::TicketCache::new()?;

        let encryption_key = if use_real_steam {
            // Try to extract real Steam encryption key
            Self::get_steam_encryption_key().unwrap_or_else(|_| {
                log::warn!("Failed to get real Steam key, using fallback");
                Self::generate_fallback_key()
            })
        } else {
            Self::generate_fallback_key()
        };

        Ok(Arc::new(Self {
            cache,
            encryption_key,
            use_real_steam,
        }))
    }

    pub fn generate_auth_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        network_identity: Option<u64>,
    ) -> Result<Vec<u8>> {
        log::debug!(
            "Generating auth ticket for SteamID: {}, AppID: {}",
            steam_id,
            app_id
        );

        if self.use_real_steam {
            self.generate_real_steam_ticket(steam_id, app_id)
        } else {
            self.generate_emulated_ticket(steam_id, app_id, network_identity)
        }
    }

    pub fn generate_encrypted_app_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        user_data: Vec<u8>,
    ) -> Result<EncryptedAppTicket> {
        log::debug!(
            "Generating encrypted app ticket for SteamID: {}, AppID: {}",
            steam_id,
            app_id
        );

        // Check cache first
        if let Some(cached) = self.cache.get_encrypted_ticket(steam_id, app_id) {
            if !Self::is_ticket_expired(&cached) {
                log::debug!("Using cached encrypted ticket");
                return Ok(cached);
            }
        }

        let ticket = if self.use_real_steam {
            self.generate_real_encrypted_ticket(steam_id, app_id, user_data.clone())?
        } else {
            self.generate_emulated_encrypted_ticket(steam_id, app_id, user_data.clone())?
        };

        // Cache the ticket
        self.cache
            .store_encrypted_ticket(steam_id, app_id, &ticket)?;

        Ok(ticket)
    }

    pub fn get_cached_encrypted_ticket(&self, steam_id: u64, app_id: u32) -> Option<Vec<u8>> {
        self.cache
            .get_encrypted_ticket(steam_id, app_id)
            .filter(|t| !Self::is_ticket_expired(t))
            .map(|t| t.ticket_data)
    }

    pub fn generate_web_api_ticket(&self, steam_id: u64, identity: &str) -> Result<Vec<u8>> {
        log::debug!(
            "Generating web API ticket for SteamID: {}, Identity: {}",
            steam_id,
            identity
        );

        if self.use_real_steam {
            // Use real Steam WebAPI authentication
            self.generate_real_web_ticket(steam_id, identity)
        } else {
            // Generate emulated web ticket
            self.generate_emulated_web_ticket(steam_id, identity)
        }
    }

    pub fn validate_auth_ticket(&self, ticket_data: &[u8], expected_steam_id: u64) -> Result<bool> {
        if ticket_data.len() < 24 {
            return Ok(false);
        }

        // Parse ticket header
        let ticket_version = u32::from_le_bytes(ticket_data[0..4].try_into()?);
        let steam_id = u64::from_le_bytes(ticket_data[4..12].try_into()?);
        let app_id = u32::from_le_bytes(ticket_data[12..16].try_into()?);
        let timestamp = u32::from_le_bytes(ticket_data[16..20].try_into()?);

        // Validate version
        if ticket_version != TICKET_VERSION {
            log::warn!("Invalid ticket version: {}", ticket_version);
            return Ok(false);
        }

        // Validate SteamID
        if steam_id != expected_steam_id {
            log::warn!("SteamID mismatch: {} != {}", steam_id, expected_steam_id);
            return Ok(false);
        }

        // Validate timestamp (tickets valid for 24 hours)
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

        if current_time > timestamp + 86400 {
            log::warn!("Ticket expired");
            return Ok(false);
        }

        // Validate signature
        let signature_offset = ticket_data.len() - 20;
        let signature = &ticket_data[signature_offset..];
        let data = &ticket_data[..signature_offset];

        let computed_signature = self.compute_ticket_signature(data);

        if signature != &computed_signature[..20] {
            log::warn!("Invalid ticket signature");
            return Ok(false);
        }

        Ok(true)
    }

    fn generate_real_steam_ticket(&self, steam_id: u64, app_id: u32) -> Result<Vec<u8>> {
        // This would call the real Steam API via steamworks-sys
        unsafe {
            std::env::set_var("SteamAppId", app_id.to_string());

            if steamworks_sys::SteamAPI_InitFlat(std::ptr::null_mut())
                != steamworks_sys::ESteamAPIInitResult::k_ESteamAPIInitResult_OK
            {
                bail!("Failed to initialize Steam API for ticket generation");
            }

            let user = steamworks_sys::SteamAPI_SteamUser_v023();

            let mut ticket = vec![0u8; 1024];
            let mut ticket_len = 0u32;

            let handle = steamworks_sys::SteamAPI_ISteamUser_GetAuthSessionTicket(
                user,
                ticket.as_mut_ptr() as *mut _,
                1024,
                &mut ticket_len,
                std::ptr::null(),
            );

            if handle == 0 {
                bail!("Failed to get auth session ticket from Steam");
            }

            ticket.truncate(ticket_len as usize);
            Ok(ticket)
        }
    }

    fn generate_emulated_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        network_identity: Option<u64>,
    ) -> Result<Vec<u8>> {
        let mut ticket = Vec::new();

        // Ticket version
        ticket.extend_from_slice(&TICKET_VERSION.to_le_bytes());

        // SteamID
        ticket.extend_from_slice(&steam_id.to_le_bytes());

        // AppID
        ticket.extend_from_slice(&app_id.to_le_bytes());

        // Timestamp
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        ticket.extend_from_slice(&timestamp.to_le_bytes());

        // Session ID (random)
        let session_id: u32 = rand::thread_rng().gen();
        ticket.extend_from_slice(&session_id.to_le_bytes());

        // Network identity (optional)
        if let Some(net_id) = network_identity {
            ticket.extend_from_slice(&net_id.to_le_bytes());
        } else {
            ticket.extend_from_slice(&0u64.to_le_bytes());
        }

        // External IP (simulated)
        ticket.extend_from_slice(&0u32.to_le_bytes());

        // Flags
        ticket.extend_from_slice(&0u32.to_le_bytes());

        // GC token (game coordinator)
        ticket.extend_from_slice(&0u64.to_le_bytes());

        // Signature
        let signature = self.compute_ticket_signature(&ticket);
        ticket.extend_from_slice(&signature[..20]);

        Ok(ticket)
    }

    fn generate_real_encrypted_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        user_data: Vec<u8>,
    ) -> Result<EncryptedAppTicket> {
        unsafe {
            std::env::set_var("SteamAppId", app_id.to_string());

            if steamworks_sys::SteamAPI_InitFlat(std::ptr::null_mut())
                != steamworks_sys::ESteamAPIInitResult::k_ESteamAPIInitResult_OK
            {
                bail!("Failed to initialize Steam API");
            }

            let user = steamworks_sys::SteamAPI_SteamUser_v023();

            // Request encrypted app ticket
            steamworks_sys::SteamAPI_ISteamUser_RequestEncryptedAppTicket(
                user,
                user_data.as_ptr() as *mut _,
                user_data.len() as i32,
            );

            // Wait for callback (simplified - real implementation uses callbacks)
            std::thread::sleep(std::time::Duration::from_secs(2));

            let mut ticket = vec![0u8; 2048];
            let mut ticket_len = 0u32;

            let success = steamworks_sys::SteamAPI_ISteamUser_GetEncryptedAppTicket(
                user,
                ticket.as_mut_ptr() as *mut _,
                2048,
                &mut ticket_len,
            );

            if !success {
                bail!("Failed to get encrypted app ticket from Steam");
            }

            ticket.truncate(ticket_len as usize);

            Ok(EncryptedAppTicket {
                ticket_data: ticket,
                steam_id,
                app_id,
                user_data,
                created_at: SystemTime::now(),
            })
        }
    }

    fn generate_emulated_encrypted_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        user_data: Vec<u8>,
    ) -> Result<EncryptedAppTicket> {
        // Build ticket structure
        let mut ticket_plaintext = Vec::new();

        // Ticket header
        ticket_plaintext.extend_from_slice(&TICKET_VERSION.to_le_bytes());
        ticket_plaintext.extend_from_slice(&steam_id.to_le_bytes());
        ticket_plaintext.extend_from_slice(&app_id.to_le_bytes());

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        ticket_plaintext.extend_from_slice(&timestamp.to_le_bytes());

        // User data length and content
        ticket_plaintext.extend_from_slice(&(user_data.len() as u32).to_le_bytes());
        ticket_plaintext.extend_from_slice(&user_data);

        // Ownership ticket data
        let ownership = self.generate_ownership_section(steam_id, app_id)?;
        ticket_plaintext.extend_from_slice(&ownership);

        // Encrypt the ticket
        let encrypted = self.encrypt_ticket(&ticket_plaintext)?;

        Ok(EncryptedAppTicket {
            ticket_data: encrypted,
            steam_id,
            app_id,
            user_data,
            created_at: SystemTime::now(),
        })
    }

    fn generate_ownership_section(&self, steam_id: u64, app_id: u32) -> Result<Vec<u8>> {
        let mut ownership = Vec::new();

        // License count
        ownership.extend_from_slice(&1u32.to_le_bytes());

        // License for this app
        ownership.extend_from_slice(&app_id.to_le_bytes());

        // Package ID (simulated)
        ownership.extend_from_slice(&(app_id + 1000).to_le_bytes());

        // Time created
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        ownership.extend_from_slice(&time.to_le_bytes());

        // Owner ID
        ownership.extend_from_slice(&steam_id.to_le_bytes());

        Ok(ownership)
    }

    fn generate_real_web_ticket(&self, steam_id: u64, identity: &str) -> Result<Vec<u8>> {
        // This would use Steam WebAPI to generate a real session ticket
        // For now, returning emulated version
        self.generate_emulated_web_ticket(steam_id, identity)
    }

    fn generate_emulated_web_ticket(&self, steam_id: u64, identity: &str) -> Result<Vec<u8>> {
        let mut ticket = Vec::new();

        // Session token format
        ticket.extend_from_slice(&steam_id.to_le_bytes());
        ticket.extend_from_slice(identity.as_bytes());

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        ticket.extend_from_slice(&timestamp.to_le_bytes());

        // Sign the token
        let signature = self.compute_ticket_signature(&ticket);
        ticket.extend_from_slice(&signature[..32]);

        Ok(ticket)
    }

    fn encrypt_ticket(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        use aes::cipher::{BlockEncryptMut, KeyIvInit};
        use cbc::Encryptor;

        type Aes256CbcEnc = Encryptor<Aes256>;

        // Generate random IV
        let mut iv = [0u8; 16];
        rand::thread_rng().fill(&mut iv);

        // Pad plaintext to block size
        let block_size = 16;
        let padding_len = block_size - (plaintext.len() % block_size);
        let mut padded = plaintext.to_vec();
        padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));

        // Encrypt
        let cipher = Aes256CbcEnc::new(
            GenericArray::from_slice(&self.encryption_key),
            GenericArray::from_slice(&iv),
        );

        let mut encrypted = padded.clone();
        for chunk in encrypted.chunks_mut(block_size) {
            cipher.encrypt_block_mut(GenericArray::from_mut_slice(chunk));
        }

        // Prepend IV
        let mut result = iv.to_vec();
        result.extend_from_slice(&encrypted);

        Ok(result)
    }

    fn compute_ticket_signature(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(data);
        hasher.update(&self.encryption_key);
        hasher.finalize().to_vec()
    }

    fn get_steam_encryption_key() -> Result<[u8; 32]> {
        // Try to extract from Steam client memory or config
        // This is platform-specific and complex
        bail!("Real Steam key extraction not implemented")
    }

    fn generate_fallback_key() -> [u8; 32] {
        // Generate deterministic key based on machine ID
        let mut key = [0u8; 32];
        let machine_id = oracle_core::config::get_machine_id();

        let mut hasher = sha2::Sha256::new();
        hasher.update(b"OracleSteamEncryptionKey");
        hasher.update(machine_id.as_bytes());

        let result = hasher.finalize();
        key.copy_from_slice(&result);

        key
    }

    fn is_ticket_expired(ticket: &EncryptedAppTicket) -> bool {
        if let Ok(elapsed) = ticket.created_at.elapsed() {
            elapsed.as_secs() > 86400 // 24 hours
        } else {
            true
        }
    }
}

impl Clone for TicketGenerator {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            encryption_key: self.encryption_key,
            use_real_steam: self.use_real_steam,
        }
    }
}
