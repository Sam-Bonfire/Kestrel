use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::Account;
use crate::core::repository::AccountRepository;

pub struct SqliteAccountRepository {
    pool: SqlitePool,
    master_key: String,
}

impl SqliteAccountRepository {
    pub fn new(pool: SqlitePool, master_key: String) -> Self {
        Self { pool, master_key }
    }
}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, sqlx::Error> {
        let mut account = sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
             FROM accounts WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut acc) = account {
            if let Some(ref at) = acc.access_token {
                acc.access_token = Some(
                    crate::core::crypto::decrypt(at, &self.master_key)
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                );
            }
            if let Some(ref rt) = acc.refresh_token {
                acc.refresh_token = Some(
                    crate::core::crypto::decrypt(rt, &self.master_key)
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                );
            }
        }

        Ok(account)
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>, sqlx::Error> {
        let mut accounts = sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
             FROM accounts WHERE user_id = ?",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        for acc in accounts.iter_mut() {
            if let Some(ref at) = acc.access_token {
                acc.access_token = Some(
                    crate::core::crypto::decrypt(at, &self.master_key)
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                );
            }
            if let Some(ref rt) = acc.refresh_token {
                acc.refresh_token = Some(
                    crate::core::crypto::decrypt(rt, &self.master_key)
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                );
            }
        }

        Ok(accounts)
    }

    async fn find_by_provider_account_id(
        &self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, sqlx::Error> {
        sqlx::query_as::<_, Account>(
            "SELECT id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
             FROM accounts WHERE provider = ? AND provider_account_id = ?",
        )
        .bind(provider)
        .bind(provider_account_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, account: &Account) -> Result<(), sqlx::Error> {
        let enc_access_token = match &account.access_token {
            Some(at) => Some(
                crate::core::crypto::encrypt(at, &self.master_key)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            ),
            None => None,
        };
        let enc_refresh_token = match &account.refresh_token {
            Some(rt) => Some(
                crate::core::crypto::encrypt(rt, &self.master_key)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            ),
            None => None,
        };

        sqlx::query(
            "INSERT INTO accounts (id, user_id, provider, provider_account_id, display_name, \
             access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(account.id.to_string())
        .bind(account.user_id.to_string())
        .bind(&account.provider)
        .bind(&account.provider_account_id)
        .bind(&account.display_name)
        .bind(enc_access_token)
        .bind(enc_refresh_token)
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
        let enc_access_token = match access_token {
            Some(at) => Some(
                crate::core::crypto::encrypt(at, &self.master_key)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            ),
            None => None,
        };

        let updated_at = chrono::Utc::now().timestamp();
        sqlx::query(
            "UPDATE accounts SET access_token = ?, token_expires_at = ?, sync_error = ?, updated_at = ? WHERE id = ?"
        )
        .bind(enc_access_token)
        .bind(expires_at)
        .bind(error)
        .bind(updated_at)
        .bind(id.to_string())
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
