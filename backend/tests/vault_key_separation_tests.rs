use backend::core::models::Account;
use backend::core::repository::AccountRepository;
use backend::core::types::DbUuid;
use backend::db::sqlite::account_repository::SqliteAccountRepository;
use sqlx::SqlitePool;
use uuid::Uuid;

mod common;

async fn setup_account_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE accounts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_account_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            access_token TEXT,
            refresh_token TEXT,
            token_expires_at INTEGER,
            sync_error TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn test_vault_key_independent_from_jwt_secret() {
    let pool = setup_account_db().await;
    let vault_key = "strong_vault_encryption_key_alpha!";
    let jwt_secret = "separate_session_jwt_signing_key!!";

    let repo_vault = SqliteAccountRepository::new(pool.clone(), vault_key.to_string());

    let user_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "google".to_string(),
        provider_account_id: "google-123".to_string(),
        display_name: "Test User".to_string(),
        access_token: Some("secret_access_oauth_token".to_string()),
        refresh_token: Some("secret_refresh_oauth_token".to_string()),
        token_expires_at: Some(1800000000),
        sync_error: None,
        created_at: 1000,
        updated_at: 1000,
    };

    repo_vault.create(&account).await.unwrap();

    // Reading with correct vault key decrypts both tokens
    let found = repo_vault.find_by_id(account_id).await.unwrap().unwrap();
    assert_eq!(found.access_token.as_deref(), Some("secret_access_oauth_token"));
    assert_eq!(found.refresh_token.as_deref(), Some("secret_refresh_oauth_token"));
    assert!(found.sync_error.is_none());

    // Reading with JWT secret (simulating key confusion / leak) cannot decrypt tokens
    let repo_jwt = SqliteAccountRepository::new(pool.clone(), jwt_secret.to_string());
    let compromised_read = repo_jwt.find_by_id(account_id).await.unwrap().unwrap();
    assert_eq!(compromised_read.access_token, None);
    assert_eq!(compromised_read.refresh_token, None);
    assert_eq!(compromised_read.sync_error.as_deref(), Some("reauth_required"));
}

#[tokio::test]
async fn test_server_restart_with_same_vault_key_survives() {
    let pool = setup_account_db().await;
    let vault_key = "persistent_vault_master_key_42!!";

    let account_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "outlook".to_string(),
        provider_account_id: "ms-999".to_string(),
        display_name: "Restart User".to_string(),
        access_token: Some("surviving_token_xyz".to_string()),
        refresh_token: Some("surviving_refresh_xyz".to_string()),
        token_expires_at: Some(1900000000),
        sync_error: None,
        created_at: 1000,
        updated_at: 1000,
    };

    // Server run 1
    {
        let repo_boot_1 = SqliteAccountRepository::new(pool.clone(), vault_key.to_string());
        repo_boot_1.create(&account).await.unwrap();
    }

    // Server reboot (new instance with same persistent vault key)
    {
        let repo_boot_2 = SqliteAccountRepository::new(pool.clone(), vault_key.to_string());
        let loaded = repo_boot_2.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(loaded.access_token.as_deref(), Some("surviving_token_xyz"));
        assert_eq!(loaded.refresh_token.as_deref(), Some("surviving_refresh_xyz"));
        assert_eq!(loaded.sync_error, None);
    }
}

#[tokio::test]
async fn test_server_restart_with_wrong_key_degrades_to_reauth_without_500() {
    let pool = setup_account_db().await;
    let vault_key_a = "vault_key_instance_a_1234567890!";
    let vault_key_b = "vault_key_instance_b_different!!";

    let account_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "google".to_string(),
        provider_account_id: "google-restart".to_string(),
        display_name: "Reauth User".to_string(),
        access_token: Some("valuable_oauth_secret".to_string()),
        refresh_token: Some("valuable_refresh_secret".to_string()),
        token_expires_at: Some(1800000000),
        sync_error: None,
        created_at: 1000,
        updated_at: 1000,
    };

    let repo_a = SqliteAccountRepository::new(pool.clone(), vault_key_a.to_string());
    repo_a.create(&account).await.unwrap();

    // Server reboot with different key (e.g. unpersisted random boot key)
    let repo_b = SqliteAccountRepository::new(pool.clone(), vault_key_b.to_string());
    let result = repo_b.find_by_id(account_id).await;

    // Must NOT return sqlx::Error / 500 internal error!
    assert!(result.is_ok());
    let loaded = result.unwrap().unwrap();
    assert_eq!(loaded.access_token, None);
    assert_eq!(loaded.refresh_token, None);
    assert_eq!(loaded.sync_error.as_deref(), Some("reauth_required"));
}

#[tokio::test]
async fn test_dual_read_rotation_with_fallback_key() {
    let pool = setup_account_db().await;
    let old_key = "vault_key_2025_rotation_old_key!";
    let new_key = "vault_key_2026_rotation_new_key!";

    let account_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "caldav".to_string(),
        provider_account_id: "caldav-1".to_string(),
        display_name: "Rotating Account".to_string(),
        access_token: Some("token_before_rotation".to_string()),
        refresh_token: None,
        token_expires_at: Some(1800000000),
        sync_error: None,
        created_at: 1000,
        updated_at: 1000,
    };

    // Stored under old_key
    let repo_old = SqliteAccountRepository::new(pool.clone(), old_key.to_string());
    repo_old.create(&account).await.unwrap();

    // Rotation phase: repo uses new_key with old_key fallback
    let repo_dual = SqliteAccountRepository::new(pool.clone(), new_key.to_string())
        .with_fallback_key(Some(old_key.to_string()));

    let loaded = repo_dual.find_by_id(account_id).await.unwrap().unwrap();
    assert_eq!(loaded.access_token.as_deref(), Some("token_before_rotation"));
    assert!(loaded.sync_error.is_none());

    // Update tokens with new credentials during rotation window
    repo_dual
        .update_tokens_and_error(account_id, Some("token_after_rotation"), Some(1900000000), None)
        .await
        .unwrap();

    // Rotation complete: drop fallback key, now only new_key is used
    let repo_new_only = SqliteAccountRepository::new(pool.clone(), new_key.to_string());
    let re_read = repo_new_only.find_by_id(account_id).await.unwrap().unwrap();
    assert_eq!(re_read.access_token.as_deref(), Some("token_after_rotation"));
    assert!(re_read.sync_error.is_none());
}

#[tokio::test]
async fn test_unencrypted_legacy_tokens_reject_silent_plaintext() {
    let pool = setup_account_db().await;
    let vault_key = "vault_key_strict_no_plaintext!!";

    let account_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Directly insert legacy unencrypted plaintext token into DB
    sqlx::query(
        "INSERT INTO accounts (id, user_id, provider, provider_account_id, display_name, \
         access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(account_id.to_string())
    .bind(user_id.to_string())
    .bind("google")
    .bind("google-unenc")
    .bind("Unencrypted Account")
    .bind("raw_plaintext_legacy_access_token")
    .bind("raw_plaintext_legacy_refresh_token")
    .bind(1800000000i64)
    .bind(None::<String>)
    .bind(1000i64)
    .bind(1000i64)
    .execute(&pool)
    .await
    .unwrap();

    let repo = SqliteAccountRepository::new(pool, vault_key.to_string());
    let account = repo.find_by_id(account_id).await.unwrap().unwrap();

    // Never return silent plaintext!
    assert_eq!(account.access_token, None);
    assert_eq!(account.refresh_token, None);
    assert_eq!(account.sync_error.as_deref(), Some("reauth_required"));
}
