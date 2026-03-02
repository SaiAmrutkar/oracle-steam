use super::EncryptedAppTicket;
use aes::cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit};
use aes::Aes256;
use anyhow::{bail, Result};
use sha1::{Digest, Sha1};

pub struct TicketValidator {
    encryption_key: [u8; 32],
}

impl TicketValidator {
    pub fn new(encryption_key: [u8; 32]) -> Self {
        Self { encryption_key }
    }

    pub fn validate_encrypted_ticket(
        &self,
        ticket_data: &[u8],
        expected_steam_id: u64,
        expected_app_id: u32,
    ) -> Result<bool> {
        if ticket_data.len() < 16 {
            return Ok(false);
        }

        // Decrypt ticket
        let decrypted = self.decrypt_ticket(ticket_data)?;

        // Parse ticket
        if decrypted.len() < 20 {
            return Ok(false);
        }

        let version = u32::from_le_bytes(decrypted[0..4].try_into()?);
        let steam_id = u64::from_le_bytes(decrypted[4..12].try_into()?);
        let app_id = u32::from_le_bytes(decrypted[12..16].try_into()?);
        let timestamp = u32::from_le_bytes(decrypted[16..20].try_into()?);

        // Validate version
        if version != 4 {
            log::warn!("Invalid encrypted ticket version: {}", version);
            return Ok(false);
        }

        // Validate SteamID
        if steam_id != expected_steam_id {
            log::warn!("SteamID mismatch in encrypted ticket");
            return Ok(false);
        }

        // Validate AppID
        if app_id != expected_app_id {
            log::warn!("AppID mismatch in encrypted ticket");
            return Ok(false);
        }

        // Validate timestamp
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as u32;

        if current_time > timestamp + 86400 {
            log::warn!("Encrypted ticket expired");
            return Ok(false);
        }

        Ok(true)
    }

    pub fn parse_encrypted_ticket(&self, ticket_data: &[u8]) -> Result<EncryptedAppTicket> {
        let decrypted = self.decrypt_ticket(ticket_data)?;

        if decrypted.len() < 24 {
            bail!("Ticket too short");
        }

        let version = u32::from_le_bytes(decrypted[0..4].try_into()?);
        let steam_id = u64::from_le_bytes(decrypted[4..12].try_into()?);
        let app_id = u32::from_le_bytes(decrypted[12..16].try_into()?);
        let timestamp = u32::from_le_bytes(decrypted[16..20].try_into()?);
        let user_data_len = u32::from_le_bytes(decrypted[20..24].try_into()?) as usize;

        let user_data = if user_data_len > 0 && decrypted.len() >= 24 + user_data_len {
            decrypted[24..24 + user_data_len].to_vec()
        } else {
            Vec::new()
        };

        Ok(EncryptedAppTicket {
            ticket_data: ticket_data.to_vec(),
            steam_id,
            app_id,
            user_data,
            created_at: std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64),
        })
    }

    fn decrypt_ticket(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        use aes::cipher::{BlockDecryptMut, KeyIvInit};
        use cbc::Decryptor;

        type Aes256CbcDec = Decryptor<Aes256>;

        if encrypted.len() < 16 {
            bail!("Encrypted data too short");
        }

        // Extract IV
        let iv = &encrypted[0..16];
        let ciphertext = &encrypted[16..];

        if ciphertext.len() % 16 != 0 {
            bail!("Invalid ciphertext length");
        }

        // Decrypt
        let cipher = Aes256CbcDec::new(
            GenericArray::from_slice(&self.encryption_key),
            GenericArray::from_slice(iv),
        );

        let mut decrypted = ciphertext.to_vec();
        for chunk in decrypted.chunks_mut(16) {
            cipher.decrypt_block_mut(GenericArray::from_mut_slice(chunk));
        }

        // Remove padding
        if let Some(&padding_len) = decrypted.last() {
            if padding_len as usize <= 16 && padding_len as usize <= decrypted.len() {
                decrypted.truncate(decrypted.len() - padding_len as usize);
            }
        }

        Ok(decrypted)
    }

    pub fn validate_signature(&self, data: &[u8], signature: &[u8]) -> bool {
        let computed = self.compute_signature(data);

        if signature.len() < 20 {
            return false;
        }

        &computed[..20] == &signature[..20]
    }

    fn compute_signature(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(data);
        hasher.update(&self.encryption_key);
        hasher.finalize().to_vec()
    }
}
