use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::UserPreferences;
use crate::core::repository::UserPreferencesRepository;

pub struct SqliteUserPreferencesRepository {
    pool: SqlitePool,
}

impl SqliteUserPreferencesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserPreferencesRepository for SqliteUserPreferencesRepository {
    async fn get_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>, sqlx::Error> {
        sqlx::query_as::<_, UserPreferences>(
            "SELECT user_id, preferences_json, updated_at FROM user_preferences WHERE user_id = ?",
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_preferences(
        &self,
        user_id: Uuid,
        preferences_json: &str,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO user_preferences (user_id, preferences_json, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id) DO UPDATE SET
                preferences_json = excluded.preferences_json,
                updated_at = excluded.updated_at",
        )
        .bind(user_id.to_string())
        .bind(preferences_json)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
