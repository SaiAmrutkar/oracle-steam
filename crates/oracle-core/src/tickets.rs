// crates/oracle-core/src/ticket.rs
use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes256;
use sha1::{Digest, Sha1};

const STEAM_TICKET_VERSION: u32 = 1;

#[repr(C, packed)]
struct TicketHeader {
    size: u32,
    version: u32,
    steam_id: u64,
    app_id: u32,
    external_ip: u32,
    internal_ip: u32,
    flags: u32,
    timestamp: u32,
    ticket_sequence: u32,
}

#[repr(C, packed)]
struct AppOwnershipTicket {
    ticket_version: u32,
    steam_id: u64,
    app_id: u32,
    external_ip: u32,
    internal_ip: u32,
    ownership_flags: u32,
    created: u32,
    expires: u32,
}

pub struct TicketGenerator {
    session_key: [u8; 32],
    app_key: [u8; 32],
}

impl TicketGenerator {
    pub fn new() -> Self {
        Self {
            session_key: Self::derive_session_key(),
            app_key: Self::derive_app_key(),
        }
    }

    fn derive_session_key() -> [u8; 32] {
        // Use Steam's key derivation
        let mut hasher = Sha1::new();
        hasher.update(b"Steam Session Key v1");
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key[..20].copy_from_slice(&hash);
        key[20..].copy_from_slice(&hash[..12]);
        key
    }

    fn derive_app_key() -> [u8; 32] {
        let mut hasher = Sha1::new();
        hasher.update(b"Steam App Key v1");
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key[..20].copy_from_slice(&hash);
        key[20..].copy_from_slice(&hash[..12]);
        key
    }

    pub fn generate_encrypted_ticket(
        &self,
        steam_id: u64,
        app_id: u32,
        user_data: &[u8],
    ) -> Vec<u8> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        // Build ownership ticket
        let ownership = AppOwnershipTicket {
            ticket_version: 1,
            steam_id,
            app_id,
            external_ip: 0,
            internal_ip: 0,
            ownership_flags: 0xFFFFFFFF, // Full ownership
            created: timestamp,
            expires: timestamp + (30 * 24 * 60 * 60), // 30 days
        };

        // Serialize ownership ticket
        let ownership_bytes = unsafe {
            std::slice::from_raw_parts(
                &ownership as *const _ as *const u8,
                std::mem::size_of::<AppOwnershipTicket>(),
            )
        };

        // Build full ticket
        let mut ticket_data = Vec::new();
        ticket_data.extend_from_slice(ownership_bytes);
        ticket_data.extend_from_slice(user_data);

        // Encrypt with AES-256
        let cipher = Aes256::new(GenericArray::from_slice(&self.app_key));

        // Pad to block size
        let padding = (16 - (ticket_data.len() % 16)) % 16;
        ticket_data.extend(std::iter::repeat(padding as u8).take(padding));

        // Encrypt in-place
        for chunk in ticket_data.chunks_mut(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            chunk.copy_from_slice(&block);
        }

        // Build header
        let header = TicketHeader {
            size: (std::mem::size_of::<TicketHeader>() + ticket_data.len()) as u32,
            version: STEAM_TICKET_VERSION,
            steam_id,
            app_id,
            external_ip: 0,
            internal_ip: 0,
            flags: 0,
            timestamp,
            ticket_sequence: rand::random(),
        };

        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<TicketHeader>(),
            )
        };

        let mut result = Vec::new();
        result.extend_from_slice(header_bytes);
        result.extend_from_slice(&ticket_data);

        result
    }

    pub fn validate_ticket(&self, ticket: &[u8]) -> Result<(u64, u32), String> {
        if ticket.len() < std::mem::size_of::<TicketHeader>() {
            return Err("Ticket too short".into());
        }

        let header = unsafe { &*(ticket.as_ptr() as *const TicketHeader) };

        if header.version != STEAM_TICKET_VERSION {
            return Err("Invalid ticket version".into());
        }

        let encrypted_data = &ticket[std::mem::size_of::<TicketHeader>()..];

        // Decrypt
        let cipher = Aes256::new(GenericArray::from_slice(&self.app_key));
        let mut decrypted = encrypted_data.to_vec();

        for chunk in decrypted.chunks_mut(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block); // AES decrypt = encrypt with inverse
            chunk.copy_from_slice(&block);
        }

        // Parse ownership ticket
        let ownership = unsafe { &*(decrypted.as_ptr() as *const AppOwnershipTicket) };

        Ok((ownership.steam_id, ownership.app_id))
    }
}
