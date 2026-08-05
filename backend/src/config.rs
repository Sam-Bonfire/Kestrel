use tracing::warn;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub bind_addr: String,
    pub gmail_client_id: Option<String>,
    pub gmail_client_secret: Option<String>,
    pub outlook_client_id: Option<String>,
    pub outlook_client_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| format!("sqlite:{}/data/kestrel.db", manifest_dir));
        let jwt_secret = Self::resolve_jwt_secret();
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        let gmail_client_id = std::env::var("GMAIL_CLIENT_ID").ok();
        let gmail_client_secret = std::env::var("GMAIL_CLIENT_SECRET").ok();
        let outlook_client_id = std::env::var("OUTLOOK_CLIENT_ID").ok();
        let outlook_client_secret = std::env::var("OUTLOOK_CLIENT_SECRET").ok();

        Config {
            database_url,
            jwt_secret,
            bind_addr,
            gmail_client_id,
            gmail_client_secret,
            outlook_client_id,
            outlook_client_secret,
        }
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

        warn!("JWT_SECRET not set — using random secret (tokens will not survive restarts)");
        secret
    }
}
