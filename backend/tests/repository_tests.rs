mod common;

use backend::core::models::{Account, User};
use backend::core::repository::{AccountRepository, UserRepository};
use backend::core::types::DbUuid;
use backend::db::pool::DbPool;
use backend::db::sqlite::account_repository::SqliteAccountRepository;
use backend::db::sqlite::user_repository::SqliteUserRepository;
use common::setup_test_db;
use uuid::Uuid;

fn get_sqlite_pool(pool: &DbPool) -> sqlx::SqlitePool {
    match pool {
        DbPool::Sqlite(p) => p.clone(),
        _ => panic!("Expected SQLite pool"),
    }
}

#[tokio::test]
async fn test_user_repository_crud() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let repo = SqliteUserRepository::new(sqlite_pool);

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let user = User {
        id: DbUuid::from(user_id),
        username: "test@kestrel.dev".to_string(),
        password_hash: "argon2_hash_placeholder".to_string(),
        created_at: now,
        updated_at: now,
    };

    repo.create(&user).await.expect("User creation failed");

    let fetched = repo
        .find_by_username("test@kestrel.dev")
        .await
        .expect("Find by username failed");
    assert!(fetched.is_some());
    let created_user = fetched.unwrap();
    assert_eq!(created_user.username, "test@kestrel.dev");


}

#[tokio::test]
async fn test_account_repository_crud() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let user_repo = SqliteUserRepository::new(sqlite_pool.clone());
    let account_repo = SqliteAccountRepository::new(sqlite_pool);

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let user_model = User {
        id: DbUuid::from(user_id),
        username: "account_owner@kestrel.dev".to_string(),
        password_hash: "hash".to_string(),
        created_at: now,
        updated_at: now,
    };
    user_repo
        .create(&user_model)
        .await
        .expect("User creation failed");

    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid::from(account_id),
        user_id: DbUuid::from(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google_12345".to_string(),
        display_name: "Owner".to_string(),
        access_token: Some("access_token_123".to_string()),
        refresh_token: Some("refresh_token_123".to_string()),
        token_expires_at: None,
        created_at: now,
        updated_at: now,
    };

    account_repo
        .create(&account)
        .await
        .expect("Account creation failed");

    let list = account_repo
        .find_by_user_id(user_id)
        .await
        .expect("Account list failed");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].display_name, "Owner");
    assert_eq!(list[0].provider, "gmail");

    account_repo
        .delete(account_id)
        .await
        .expect("Account delete failed");
    let after_del = account_repo
        .find_by_user_id(user_id)
        .await
        .expect("Account list failed");
    assert_eq!(after_del.len(), 0);
}
