use sea_orm::entity::prelude::*;
use loco_rs::prelude::*;
pub use super::_entities::seller_verifications::{self, ActiveModel, Entity, Model};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

pub type SellerVerifications = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

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
