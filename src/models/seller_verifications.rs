use loco_rs::prelude::*;
pub use super::_entities::seller_verifications::{ActiveModel, Entity, Model, VerificationStatus};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

// impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn encrypt_data(data: &[u8], key_str: &str) -> Result<String> {
        let key_bytes = key_str.as_bytes();
        // Ensure key is 32 bytes for AES-256
        let mut key_fixed = [0u8; 32];
        let len = key_bytes.len().min(32);
        key_fixed[..len].copy_from_slice(&key_bytes[..len]);

        let cipher = Aes256Gcm::new_from_slice(&key_fixed)
            .map_err(|e| Error::BadRequest(format!("Encryption error: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| Error::BadRequest(format!("Encryption error: {}", e)))?;

        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(combined))
    }

    pub fn decrypt_data(encrypted_base64: &str, key_str: &str) -> Result<Vec<u8>> {
        let combined = general_purpose::STANDARD
            .decode(encrypted_base64)
            .map_err(|e| Error::BadRequest(format!("Decode error: {}", e)))?;

        if combined.len() < 12 {
            return Err(Error::BadRequest("Invalid encrypted data".into()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key_bytes = key_str.as_bytes();
        let mut key_fixed = [0u8; 32];
        let len = key_bytes.len().min(32);
        key_fixed[..len].copy_from_slice(&key_bytes[..len]);

        let cipher = Aes256Gcm::new_from_slice(&key_fixed)
            .map_err(|e| Error::BadRequest(format!("Decryption error: {}", e)))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::BadRequest(format!("Decryption error: {}", e)))?;

        Ok(plaintext)
    }
}
