use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};

use crate::core::error::{KestrelError, SimpleError};

#[allow(deprecated)]
pub fn encrypt(plaintext: &str, master_key: &str) -> Result<String, KestrelError> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let mut hasher = Sha256::new();
    hasher.update(master_key.as_bytes());
    let result = hasher.finalize();
    let key = Key::<Aes256Gcm>::from_slice(result.as_slice());

    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| KestrelError::Internal(Box::new(SimpleError(format!("Encryption failed: {}", e)))))?;

    let nonce_b64 = STANDARD.encode(nonce);
    let ciphertext_b64 = STANDARD.encode(ciphertext);

    Ok(format!("v1:{}:{}", nonce_b64, ciphertext_b64))
}

#[allow(deprecated)]
pub fn decrypt(encrypted: &str, master_key: &str) -> Result<String, KestrelError> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    if !encrypted.starts_with("v1:") {
        // Fallback for unencrypted tokens (e.g. from before encryption was added)
        return Ok(encrypted.to_string());
    }

    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 3 {
        return Err(KestrelError::Internal(Box::new(SimpleError("Invalid encrypted format".to_string()))));
    }

    let nonce_b64 = parts[1];
    let ciphertext_b64 = parts[2];

    let nonce_bytes = STANDARD.decode(nonce_b64)
        .map_err(|e| KestrelError::Internal(Box::new(SimpleError(format!("Base64 decode failed for nonce: {}", e)))))?;
    let ciphertext_bytes = STANDARD.decode(ciphertext_b64)
        .map_err(|e| KestrelError::Internal(Box::new(SimpleError(format!("Base64 decode failed for ciphertext: {}", e)))))?;

    let mut hasher = Sha256::new();
    hasher.update(master_key.as_bytes());
    let result = hasher.finalize();
    let key = Key::<Aes256Gcm>::from_slice(result.as_slice());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes.as_slice());

    let plaintext = cipher
        .decrypt(nonce, ciphertext_bytes.as_ref())
        .map_err(|e| KestrelError::Internal(Box::new(SimpleError(format!("Decryption failed: {}", e)))))?;

    String::from_utf8(plaintext)
        .map_err(|e| KestrelError::Internal(Box::new(SimpleError(format!("Invalid UTF-8 in decrypted string: {}", e)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let master_key = "super_secret_session_key";
        let plaintext = "oauth_token_12345";

        let encrypted = encrypt(plaintext, master_key).unwrap();

        assert!(encrypted.starts_with("v1:"));
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt(&encrypted, master_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_unencrypted_fallback() {
        let master_key = "super_secret_session_key";
        let unencrypted = "old_plaintext_token";

        let decrypted = decrypt(unencrypted, master_key).unwrap();
        assert_eq!(decrypted, unencrypted);
    }

    #[test]
    fn test_empty_string() {
        let master_key = "super_secret_session_key";
        let plaintext = "";

        let encrypted = encrypt(plaintext, master_key).unwrap();
        assert_eq!(encrypted, "");

        let decrypted = decrypt(&encrypted, master_key).unwrap();
        assert_eq!(decrypted, "");
    }
}
