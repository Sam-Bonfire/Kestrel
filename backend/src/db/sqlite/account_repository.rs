use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::Account;
use crate::core::repository::AccountRepository;

pub struct SqliteAccountRepository {
    pool: SqlitePool,
}

impl SqliteAccountRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, created_at, updated_at \
             FROM accounts WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, created_at, updated_at \
             FROM accounts WHERE user_id = ?",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, account: &Account) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO accounts (id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account.id.to_string())
        .bind(account.user_id.to_string())
        .bind(&account.provider)
        .bind(&account.provider_account_id)
        .bind(&account.display_name)
        .bind(&account.access_token)
        .bind(&account.refresh_token)
        .bind(account.token_expires_at)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
