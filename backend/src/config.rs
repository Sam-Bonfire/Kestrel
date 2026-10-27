use tracing::warn;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub vault_secret: String,
    pub vault_old_secret: Option<String>,
    pub bind_addr: String,
    /// Opt-in local crash reports (KESTREL_CRASH_LOG=1). Nothing leaves the machine.
    pub crash_log: bool,
    /// Max auth requests per minute per client (AUTH_RATE_LIMIT_MAX, default 10).
    pub auth_rate_limit_max: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self::from_env_getter(|key| std::env::var(key).ok())
    }

    pub fn from_env_getter<F>(mut get_var: F) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let database_url = get_var("DATABASE_URL")
            .unwrap_or_else(|| format!("sqlite:{}/data/kestrel.db", manifest_dir));

        let is_prod = matches!(
            get_var("KESTREL_ENV")
                .or_else(|| get_var("ENVIRONMENT"))
                .or_else(|| get_var("APP_ENV"))
                .or_else(|| get_var("NODE_ENV"))
                .as_deref(),
            Some("production") | Some("prod")
        );

        let jwt_secret = Self::resolve_jwt_secret_with(&mut get_var, is_prod);
        let vault_secret = Self::resolve_vault_secret_with(&mut get_var, is_prod);
        let vault_old_secret = get_var("VAULT_OLD_SECRET")
            .or_else(|| get_var("KESTREL_VAULT_OLD_KEY"))
            .filter(|s| !s.is_empty());

        let bind_addr = get_var("BIND_ADDR").unwrap_or_else(|| {
            let host = get_var("HOST").unwrap_or_else(|| "0.0.0.0".to_string());
            let port = get_var("PORT").unwrap_or_else(|| "8080".to_string());
            format!("{}:{}", host, port)
        });
        let crash_log = matches!(
            get_var("KESTREL_CRASH_LOG").as_deref(),
            Some("1") | Some("true")
        );
        let auth_rate_limit_max = get_var("AUTH_RATE_LIMIT_MAX")
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(10);

        Config {
            database_url,
            jwt_secret,
            vault_secret,
            vault_old_secret,
            bind_addr,
            crash_log,
            auth_rate_limit_max,
        }
    }

    fn resolve_jwt_secret_with<F>(mut get_var: F, is_prod: bool) -> String
    where
        F: FnMut(&str) -> Option<String>,
    {
        if let Some(secret) = get_var("JWT_SECRET").or_else(|| get_var("SESSION_SECRET"))
            && !secret.is_empty()
        {
            return secret;
        }

        if is_prod {
            panic!("Production mode requires JWT_SECRET or SESSION_SECRET to be explicitly configured. Failing closed.");
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
            "JWT_SECRET / SESSION_SECRET not set — using random secret (tokens will not survive restarts)"
        );
        secret
    }

    fn resolve_vault_secret_with<F>(mut get_var: F, is_prod: bool) -> String
    where
        F: FnMut(&str) -> Option<String>,
    {
        if let Some(secret) = get_var("VAULT_SECRET").or_else(|| get_var("KESTREL_VAULT_KEY"))
            && !secret.is_empty()
        {
            return secret;
        }

        if is_prod {
            panic!("Production mode requires VAULT_SECRET or KESTREL_VAULT_KEY to be explicitly configured. Failing closed.");
        }

        // Generate a random vault secret when none is configured.
        // Stored provider tokens won't survive restarts — acceptable for development.
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        warn!(
            "VAULT_SECRET / KESTREL_VAULT_KEY not set — using random secret (stored provider tokens will not survive restarts)"
        );
        secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_config_resolution_with_session_secret_and_host_port() {
        let mut envs = HashMap::new();
        envs.insert("HOST".to_string(), "127.0.0.1".to_string());
        envs.insert("PORT".to_string(), "9090".to_string());
        envs.insert(
            "SESSION_SECRET".to_string(),
            "custom_session_secret_123".to_string(),
        );
        envs.insert(
            "VAULT_SECRET".to_string(),
            "custom_vault_secret_456".to_string(),
        );

        let config = Config::from_env_getter(|k| envs.get(k).cloned());
        assert_eq!(config.bind_addr, "127.0.0.1:9090");
        assert_eq!(config.jwt_secret, "custom_session_secret_123");
        assert_eq!(config.vault_secret, "custom_vault_secret_456");
        assert_eq!(config.vault_old_secret, None);
    }

    #[test]
    fn test_config_resolution_with_bind_addr_and_jwt_secret() {
        let mut envs = HashMap::new();
        envs.insert("BIND_ADDR".to_string(), "0.0.0.0:3000".to_string());
        envs.insert("HOST".to_string(), "127.0.0.1".to_string());
        envs.insert("PORT".to_string(), "9090".to_string());
        envs.insert(
            "JWT_SECRET".to_string(),
            "custom_jwt_secret_456".to_string(),
        );
        envs.insert(
            "SESSION_SECRET".to_string(),
            "custom_session_secret_123".to_string(),
        );
        envs.insert(
            "KESTREL_VAULT_KEY".to_string(),
            "custom_vault_key_789".to_string(),
        );
        envs.insert(
            "VAULT_OLD_SECRET".to_string(),
            "old_vault_rotation_key".to_string(),
        );

        let config = Config::from_env_getter(|k| envs.get(k).cloned());
        assert_eq!(config.bind_addr, "0.0.0.0:3000");
        assert_eq!(config.jwt_secret, "custom_jwt_secret_456");
        assert_eq!(config.vault_secret, "custom_vault_key_789");
        assert_eq!(
            config.vault_old_secret,
            Some("old_vault_rotation_key".to_string())
        );
    }

    #[test]
    fn test_config_resolution_defaults() {
        let envs = HashMap::<String, String>::new();
        let config = Config::from_env_getter(|k| envs.get(k).cloned());
        assert_eq!(config.bind_addr, "0.0.0.0:8080");
        assert_eq!(config.jwt_secret.len(), 32);
        assert_eq!(config.vault_secret.len(), 32);
        assert_ne!(config.jwt_secret, config.vault_secret);
        assert_eq!(config.vault_old_secret, None);
        assert!(!config.crash_log);
        assert_eq!(config.auth_rate_limit_max, 10);
    }

    #[test]
    #[should_panic(expected = "Production mode requires JWT_SECRET or SESSION_SECRET")]
    fn test_production_fails_without_jwt_secret() {
        let mut envs = HashMap::new();
        envs.insert("ENVIRONMENT".to_string(), "production".to_string());
        envs.insert("VAULT_SECRET".to_string(), "prod_vault_key_123".to_string());

        let _ = Config::from_env_getter(|k| envs.get(k).cloned());
    }

    #[test]
    #[should_panic(expected = "Production mode requires VAULT_SECRET or KESTREL_VAULT_KEY")]
    fn test_production_fails_without_vault_secret() {
        let mut envs = HashMap::new();
        envs.insert("KESTREL_ENV".to_string(), "production".to_string());
        envs.insert("JWT_SECRET".to_string(), "prod_jwt_secret_123".to_string());

        let _ = Config::from_env_getter(|k| envs.get(k).cloned());
    }

    #[test]
    fn test_auth_rate_limit_max_override() {
        let mut envs = HashMap::new();
        envs.insert("AUTH_RATE_LIMIT_MAX".to_string(), "60".to_string());
        let config = Config::from_env_getter(|k| envs.get(k).cloned());
        assert_eq!(config.auth_rate_limit_max, 60);
    }

    #[test]
    fn test_auth_rate_limit_max_invalid_falls_back_to_default() {
        for bad in ["0", "-5", "unlimited", ""] {
            let mut envs = HashMap::new();
            envs.insert("AUTH_RATE_LIMIT_MAX".to_string(), bad.to_string());
            let config = Config::from_env_getter(|k| envs.get(k).cloned());
            assert_eq!(config.auth_rate_limit_max, 10);
        }
    }

    #[test]
    fn test_crash_log_opt_in() {
        let mut envs = HashMap::new();
        envs.insert("KESTREL_CRASH_LOG".to_string(), "1".to_string());
        let config = Config::from_env_getter(|k| envs.get(k).cloned());
        assert!(config.crash_log);
    }
}
