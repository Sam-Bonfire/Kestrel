use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::Aead,
};
use argon2::Argon2;
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::core::error::{KestrelError, SimpleError};

#[allow(deprecated)]
pub fn encrypt(plaintext: &str, master_key: &str) -> Result<String, KestrelError> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    let mut key_bytes = [0u8; 32];
    Argon2::default()
        .hash_password_into(master_key.as_bytes(), &salt_bytes, &mut key_bytes)
        .map_err(|e| {
            KestrelError::Internal(Box::new(SimpleError(format!("KDF derivation failed: {}", e))))
        })?;

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| {
        KestrelError::Internal(Box::new(SimpleError(format!("Cipher init failed: {}", e))))
    })?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|e| {
        KestrelError::Internal(Box::new(SimpleError(format!("Encryption failed: {}", e))))
    })?;

    let salt_b64 = STANDARD.encode(salt_bytes);
    let nonce_b64 = STANDARD.encode(nonce_bytes);
    let ciphertext_b64 = STANDARD.encode(ciphertext);

    Ok(format!("v2:{}:{}:{}", salt_b64, nonce_b64, ciphertext_b64))
}

#[allow(deprecated)]
pub fn decrypt(encrypted: &str, master_key: &str) -> Result<String, KestrelError> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    if encrypted.starts_with("v2:") {
        let parts: Vec<&str> = encrypted.split(':').collect();
        if parts.len() != 4 {
            return Err(KestrelError::ReauthRequired(
                "Invalid v2 encrypted format".to_string(),
            ));
        }

        let salt_bytes = STANDARD.decode(parts[1]).map_err(|_| {
            KestrelError::ReauthRequired("Invalid salt in ciphertext".to_string())
        })?;
        let nonce_bytes = STANDARD.decode(parts[2]).map_err(|_| {
            KestrelError::ReauthRequired("Invalid nonce in ciphertext".to_string())
        })?;
        let ciphertext_bytes = STANDARD.decode(parts[3]).map_err(|_| {
            KestrelError::ReauthRequired("Invalid base64 in ciphertext".to_string())
        })?;

        if nonce_bytes.len() != 12 {
            return Err(KestrelError::ReauthRequired(
                "Invalid nonce length".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        Argon2::default()
            .hash_password_into(master_key.as_bytes(), &salt_bytes, &mut key_bytes)
            .map_err(|e| {
                KestrelError::Internal(Box::new(SimpleError(format!("KDF derivation failed: {}", e))))
            })?;

        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| {
            KestrelError::Internal(Box::new(SimpleError(format!("Cipher init failed: {}", e))))
        })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext_bytes.as_ref())
            .map_err(|_| {
                KestrelError::ReauthRequired(
                    "Decryption failed: invalid key or corrupted vault payload".to_string(),
                )
            })?;

        return String::from_utf8(plaintext).map_err(|e| {
            KestrelError::Internal(Box::new(SimpleError(format!(
                "Invalid UTF-8 in decrypted string: {}",
                e
            ))))
        });
    }

    if encrypted.starts_with("v1:") {
        let parts: Vec<&str> = encrypted.split(':').collect();
        if parts.len() != 3 {
            return Err(KestrelError::ReauthRequired(
                "Invalid v1 encrypted format".to_string(),
            ));
        }

        let nonce_bytes = STANDARD.decode(parts[1]).map_err(|_| {
            KestrelError::ReauthRequired("Invalid nonce in v1 ciphertext".to_string())
        })?;
        let ciphertext_bytes = STANDARD.decode(parts[2]).map_err(|_| {
            KestrelError::ReauthRequired("Invalid base64 in v1 ciphertext".to_string())
        })?;

        if nonce_bytes.len() != 12 {
            return Err(KestrelError::ReauthRequired(
                "Invalid nonce length in v1 ciphertext".to_string(),
            ));
        }

        let mut hasher = Sha256::new();
        hasher.update(master_key.as_bytes());
        let result = hasher.finalize();
        let cipher = Aes256Gcm::new_from_slice(result.as_slice()).map_err(|e| {
            KestrelError::Internal(Box::new(SimpleError(format!("Cipher init failed: {}", e))))
        })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext_bytes.as_ref())
            .map_err(|_| {
                KestrelError::ReauthRequired(
                    "Decryption failed: invalid key or corrupted v1 vault payload".to_string(),
                )
            })?;

        return String::from_utf8(plaintext).map_err(|e| {
            KestrelError::Internal(Box::new(SimpleError(format!(
                "Invalid UTF-8 in decrypted string: {}",
                e
            ))))
        });
    }

    // Unprefixed or legacy unencrypted token: reject silent plaintext usage
    Err(KestrelError::ReauthRequired(
        "Unencrypted or unsupported vault ciphertext: re-authentication required".to_string(),
    ))
}

pub fn decrypt_with_fallback(
    encrypted: &str,
    master_key: &str,
    fallback_key: Option<&str>,
) -> Result<String, KestrelError> {
    match decrypt(encrypted, master_key) {
        Ok(decrypted) => Ok(decrypted),
        Err(err) => {
            if let Some(fallback) = fallback_key {
                decrypt(encrypted, fallback)
            } else {
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_encryption_decryption_roundtrip() {
        let master_key = "super_secret_vault_key_1";
        let plaintext = "oauth_token_12345";

        let encrypted = encrypt(plaintext, master_key).unwrap();

        assert!(encrypted.starts_with("v2:"));
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt(&encrypted, master_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    #[allow(deprecated)]
    fn test_v1_backward_compatibility() {
        let master_key = "super_secret_vault_key_v1";
        let plaintext = "legacy_token_67890";

        // Craft valid v1 ciphertext using legacy sha256 key
        let mut hasher = Sha256::new();
        hasher.update(master_key.as_bytes());
        let result = hasher.finalize();
        let cipher = Aes256Gcm::new_from_slice(result.as_slice()).unwrap();

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();

        let v1_encrypted = format!(
            "v1:{}:{}",
            STANDARD.encode(nonce_bytes),
            STANDARD.encode(ciphertext)
        );

        let decrypted = decrypt(&v1_encrypted, master_key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails_with_reauth() {
        let master_key = "correct_vault_key";
        let wrong_key = "wrong_vault_key";
        let plaintext = "sensitive_oauth_secret";

        let encrypted = encrypt(plaintext, master_key).unwrap();
        let result = decrypt(&encrypted, wrong_key);

        assert!(matches!(result, Err(KestrelError::ReauthRequired(_))));
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext_fails_with_reauth() {
        let master_key = "correct_vault_key";
        let plaintext = "sensitive_oauth_secret";

        let encrypted = encrypt(plaintext, master_key).unwrap();
        let corrupted = format!("{}corrupted", encrypted);
        let result = decrypt(&corrupted, master_key);

        assert!(matches!(result, Err(KestrelError::ReauthRequired(_))));
    }

    #[test]
    fn test_unencrypted_legacy_token_rejects_silent_plaintext() {
        let master_key = "vault_key";
        let unencrypted = "old_plaintext_token_without_prefix";

        let result = decrypt(unencrypted, master_key);
        assert!(matches!(result, Err(KestrelError::ReauthRequired(_))));
    }

    #[test]
    fn test_empty_string() {
        let master_key = "super_secret_vault_key";
        let plaintext = "";

        let encrypted = encrypt(plaintext, master_key).unwrap();
        assert_eq!(encrypted, "");

        let decrypted = decrypt(&encrypted, master_key).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_decrypt_with_fallback_key() {
        let old_key = "old_vault_key_2025";
        let new_key = "new_vault_key_2026";
        let plaintext = "rotated_oauth_token";

        // Encrypted with old key
        let encrypted_old = encrypt(plaintext, old_key).unwrap();

        // Primary key is new_key, fallback is old_key
        let decrypted = decrypt_with_fallback(&encrypted_old, new_key, Some(old_key)).unwrap();
        assert_eq!(decrypted, plaintext);

        // Fallback None with new_key fails
        let result = decrypt_with_fallback(&encrypted_old, new_key, None);
        assert!(matches!(result, Err(KestrelError::ReauthRequired(_))));
    }
}
