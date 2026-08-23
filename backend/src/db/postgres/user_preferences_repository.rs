use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::UserPreferences;
use crate::core::repository::UserPreferencesRepository;

pub struct PostgresUserPreferencesRepository {
    pool: PgPool,
}

impl PostgresUserPreferencesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserPreferencesRepository for PostgresUserPreferencesRepository {
    async fn get_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>, sqlx::Error> {
        // Since Postgres has a JSONB column and our struct expects String, we need to cast or retrieve it as text.
        sqlx::query_as::<_, UserPreferences>(
            "SELECT user_id, preferences_json::text as preferences_json, updated_at FROM user_preferences WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_preferences(
        &self,
        user_id: Uuid,
        preferences_json: &str,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        // Here we parse it back to jsonb. We bind it as a string and cast to jsonb in SQL.
        sqlx::query(
            "INSERT INTO user_preferences (user_id, preferences_json, updated_at)
             VALUES ($1, $2::jsonb, $3)
             ON CONFLICT(user_id) DO UPDATE SET
                preferences_json = EXCLUDED.preferences_json,
                updated_at = EXCLUDED.updated_at",
        )
        .bind(user_id)
        .bind(preferences_json)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
