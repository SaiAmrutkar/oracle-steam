use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;
use anyhow::Result;
use rand::rngs::OsRng;
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha1::{Digest, Sha1};

pub struct SteamCrypto;

impl SteamCrypto {
    pub fn rsa_encrypt(data: &[u8], public_key_pem: &str) -> Result<Vec<u8>> {
        let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)?;
        let mut rng = OsRng;

        let encrypted = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
        Ok(encrypted)
    }

    pub fn rsa_decrypt(data: &[u8], private_key_pem: &str) -> Result<Vec<u8>> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
        let decrypted = private_key.decrypt(Pkcs1v15Encrypt, data)?;
        Ok(decrypted)
    }

    pub fn aes_encrypt(data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>> {
        use aes::cipher::{BlockEncryptMut, KeyIvInit};
        use cbc::Encryptor;

        type Aes256CbcEnc = Encryptor<Aes256>;

        // Pad data
        let block_size = 16;
        let padding_len = block_size - (data.len() % block_size);
        let mut padded = data.to_vec();
        padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));

        let cipher = Aes256CbcEnc::new(GenericArray::from_slice(key), GenericArray::from_slice(iv));

        let mut encrypted = padded.clone();
        for chunk in encrypted.chunks_mut(block_size) {
            cipher.encrypt_block_mut(GenericArray::from_mut_slice(chunk));
        }

        Ok(encrypted)
    }

    pub fn aes_decrypt(data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>> {
        use aes::cipher::{BlockDecryptMut, KeyIvInit};
        use cbc::Decryptor;

        type Aes256CbcDec = Decryptor<Aes256>;

        if data.len() % 16 != 0 {
            anyhow::bail!("Invalid encrypted data length");
        }

        let cipher = Aes256CbcDec::new(GenericArray::from_slice(key), GenericArray::from_slice(iv));

        let mut decrypted = data.to_vec();
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

    pub fn generate_session_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    }

    pub fn hmac_sha1(data: &[u8], key: &[u8]) -> Vec<u8> {
        use hmac::{Hmac, Mac};

        type HmacSha1 = Hmac<Sha1>;

        let mut mac = HmacSha1::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    pub fn sha1_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    pub fn generate_random_bytes(length: usize) -> Vec<u8> {
        use rand::Rng;
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill(&mut bytes[..]);
        bytes
    }
}
