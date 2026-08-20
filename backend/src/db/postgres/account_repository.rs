use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::Account;
use crate::core::repository::AccountRepository;

pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
             FROM accounts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
             FROM accounts WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_provider_account_id(
        &self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
             FROM accounts WHERE provider = $1 AND provider_account_id = $2",
        )
        .bind(provider)
        .bind(provider_account_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, account: &Account) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO accounts (id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(account.id)
        .bind(account.user_id)
        .bind(&account.provider)
        .bind(&account.provider_account_id)
        .bind(&account.display_name)
        .bind(&account.access_token)
        .bind(&account.refresh_token)
        .bind(account.token_expires_at)
        .bind(&account.sync_error)
        .bind(account.created_at)
        .bind(account.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_tokens_and_error(
        &self,
        id: Uuid,
        access_token: Option<&str>,
        expires_at: Option<i64>,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let updated_at = chrono::Utc::now().timestamp();
        sqlx::query(
            "UPDATE accounts SET access_token = $1, token_expires_at = $2, sync_error = $3, updated_at = $4 WHERE id = $5"
        )
        .bind(access_token)
        .bind(expires_at)
        .bind(error)
        .bind(updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM accounts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
