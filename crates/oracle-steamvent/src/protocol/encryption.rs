// crates/oracle-steamvent/src/protocol/encryption.rs
// Complete encryption layer for Steam protocol

use anyhow::{bail, Result};
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM,
};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use std::sync::atomic::{AtomicU64, Ordering};

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Manages encryption for Steam protocol communication
pub struct CryptoContext {
    session_key: [u8; KEY_LEN],
    hmac_key: [u8; KEY_LEN],
    sequence_num: AtomicU64,
    rng: SystemRandom,
}

impl CryptoContext {
    /// Create new crypto context with random keys
    pub fn new() -> Result<Self> {
        let rng = SystemRandom::new();
        let mut session_key = [0u8; KEY_LEN];
        let mut hmac_key = [0u8; KEY_LEN];

        rng.fill(&mut session_key)
            .map_err(|_| anyhow::anyhow!("Failed to generate session key"))?;
        rng.fill(&mut hmac_key)
            .map_err(|_| anyhow::anyhow!("Failed to generate HMAC key"))?;

        Ok(Self {
            session_key,
            hmac_key,
            sequence_num: AtomicU64::new(0),
            rng,
        })
    }

    /// Create from existing session key (after handshake)
    pub fn from_session_key(session_key: [u8; KEY_LEN], hmac_key: [u8; KEY_LEN]) -> Self {
        Self {
            session_key,
            hmac_key,
            sequence_num: AtomicU64::new(0),
            rng: SystemRandom::new(),
        }
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let seq = self.sequence_num.fetch_add(1, Ordering::SeqCst);

        // Generate nonce from sequence number
        let mut nonce_bytes = [0u8; NONCE_LEN];
        nonce_bytes[..8].copy_from_slice(&seq.to_le_bytes());
        self.rng
            .fill(&mut nonce_bytes[8..])
            .map_err(|_| anyhow::anyhow!("Nonce generation failed"))?;

        // Create sealing key
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.session_key)
            .map_err(|_| anyhow::anyhow!("Failed to create key"))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut sealing_key = SealingKey::new(unbound_key, CounterNonce::new(nonce_bytes));

        // Encrypt
        let mut in_out = plaintext.to_vec();
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < NONCE_LEN {
            bail!("Ciphertext too short");
        }

        // Extract nonce
        let nonce_bytes = &ciphertext[..NONCE_LEN];
        let encrypted_data = &ciphertext[NONCE_LEN..];

        // Create opening key
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.session_key)
            .map_err(|_| anyhow::anyhow!("Failed to create key"))?;

        let nonce = Nonce::assume_unique_for_key(*nonce_bytes.first_chunk::<NONCE_LEN>().unwrap());
        let mut opening_key = OpeningKey::new(
            unbound_key,
            CounterNonce::new(*nonce_bytes.first_chunk().unwrap()),
        );

        // Decrypt
        let mut in_out = encrypted_data.to_vec();
        let plaintext = opening_key
            .open_in_place(Aad::empty(), &mut in_out)
            .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

        Ok(plaintext.to_vec())
    }

    /// Generate HMAC for message authentication
    pub fn generate_hmac(&self, data: &[u8]) -> [u8; 32] {
        use ring::hmac;

        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);
        let signature = hmac::sign(&key, data);

        let mut hmac_bytes = [0u8; 32];
        hmac_bytes.copy_from_slice(signature.as_ref());
        hmac_bytes
    }

    /// Verify HMAC
    pub fn verify_hmac(&self, data: &[u8], expected_hmac: &[u8; 32]) -> bool {
        use ring::hmac;

        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);
        hmac::verify(&key, data, expected_hmac).is_ok()
    }

    /// Get session key for handshake
    pub fn session_key(&self) -> &[u8; KEY_LEN] {
        &self.session_key
    }

    /// Get HMAC key
    pub fn hmac_key(&self) -> &[u8; KEY_LEN] {
        &self.hmac_key
    }
}

/// Nonce sequence for AES-GCM
struct CounterNonce {
    counter: [u8; NONCE_LEN],
}

impl CounterNonce {
    fn new(nonce: [u8; NONCE_LEN]) -> Self {
        Self { counter: nonce }
    }
}

impl NonceSequence for CounterNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.counter);

        // Increment counter
        for byte in self.counter.iter_mut().rev() {
            *byte = byte.wrapping_add(1);
            if *byte != 0 {
                break;
            }
        }

        Ok(nonce)
    }
}

/// RSA key exchange for initial handshake
pub struct RsaHandshake {
    rng: SystemRandom,
}

impl RsaHandshake {
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }

    /// Encrypt session key with RSA public key
    pub fn encrypt_session_key(
        &self,
        session_key: &[u8; 32],
        public_key_der: &[u8],
    ) -> Result<Vec<u8>> {
        use rsa::pkcs8::DecodePublicKey;
        use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

        // Parse public key
        let public_key = RsaPublicKey::from_public_key_der(public_key_der)
            .map_err(|e| anyhow::anyhow!("Failed to parse public key: {}", e))?;

        // Encrypt session key
        let mut rng = rand::thread_rng();
        let encrypted = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, session_key)
            .map_err(|e| anyhow::anyhow!("RSA encryption failed: {}", e))?;

        Ok(encrypted)
    }

    /// Decrypt session key with RSA private key
    pub fn decrypt_session_key(
        &self,
        encrypted: &[u8],
        private_key_der: &[u8],
    ) -> Result<[u8; 32]> {
        use rsa::pkcs8::DecodePrivateKey;
        use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

        // Parse private key
        let private_key = RsaPrivateKey::from_pkcs8_der(private_key_der)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        // Decrypt session key
        let decrypted = private_key
            .decrypt(Pkcs1v15Encrypt, encrypted)
            .map_err(|e| anyhow::anyhow!("RSA decryption failed: {}", e))?;

        if decrypted.len() != 32 {
            bail!("Invalid session key length");
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted);
        Ok(key)
    }
}

impl Default for RsaHandshake {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let crypto = CryptoContext::new().unwrap();
        let plaintext = b"Hello, Steam!";

        let encrypted = crypto.encrypt(plaintext).unwrap();
        assert_ne!(encrypted.as_slice(), plaintext);

        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_hmac() {
        let crypto = CryptoContext::new().unwrap();
        let data = b"Test message";

        let hmac = crypto.generate_hmac(data);
        assert!(crypto.verify_hmac(data, &hmac));

        // Wrong data should fail
        assert!(!crypto.verify_hmac(b"Wrong data", &hmac));
    }

    #[test]
    fn test_multiple_encryptions() {
        let crypto = CryptoContext::new().unwrap();

        for i in 0..100 {
            let plaintext = format!("Message {}", i);
            let encrypted = crypto.encrypt(plaintext.as_bytes()).unwrap();
            let decrypted = crypto.decrypt(&encrypted).unwrap();
            assert_eq!(decrypted, plaintext.as_bytes());
        }
    }
}
