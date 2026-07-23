use std::collections::HashMap;
use tracing::{info, warn};

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self::load_secretspec();

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| format!("sqlite:{}/data/kestrel.db", manifest_dir));
        let jwt_secret = Self::resolve_jwt_secret();
        let bind_addr =
            std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        Config {
            database_url,
            jwt_secret,
            bind_addr,
        }
    }

    fn load_secretspec() {
        let path = match std::env::var("SECRETSPEC_PATH") {
            Ok(p) => p,
            Err(_) => return,
        };

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read secretspec at {}: {}", path, e);
                return;
            }
        };

        let values: HashMap<String, String> = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse secretspec at {}: {}", path, e);
                return;
            }
        };

        for (key, value) in &values {
            // SAFETY: called once at startup before any threads are spawned
            unsafe {
                std::env::set_var(key, value);
            }
        }
        info!("Loaded {} secrets from {}", values.len(), path);
    }

    fn resolve_jwt_secret() -> String {
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            if !secret.is_empty() {
                return secret;
            }
        }

        // Generate a random secret when none is configured.
        // Tokens won't survive restarts — acceptable for development.
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        warn!(
            "JWT_SECRET not set — using random secret (tokens will not survive restarts)"
        );
        secret
    }
}
