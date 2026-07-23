use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::Calendar;
use crate::core::repository::CalendarRepository;

pub struct PostgresCalendarRepository {
    pool: PgPool,
}

impl PostgresCalendarRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const CALENDAR_COLUMNS: &str =
    "c.id, c.account_id, c.external_id, c.name, c.color, c.is_primary, c.created_at, c.updated_at";

#[async_trait]
impl CalendarRepository for PostgresCalendarRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Calendar>, sqlx::Error> {
        sqlx::query_as::<_, Calendar>(&format!(
            "SELECT {CALENDAR_COLUMNS} FROM calendars c WHERE c.id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Calendar>, sqlx::Error> {
        sqlx::query_as::<_, Calendar>(&format!(
            "SELECT {CALENDAR_COLUMNS} FROM calendars c WHERE c.account_id = $1 \
             ORDER BY c.is_primary DESC, c.name"
        ))
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Calendar>, sqlx::Error> {
        sqlx::query_as::<_, Calendar>(&format!(
            "SELECT {CALENDAR_COLUMNS} \
             FROM calendars c \
             JOIN accounts a ON c.account_id = a.id \
             WHERE a.user_id = $1 \
             ORDER BY c.is_primary DESC, c.name"
        ))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert(&self, calendar: &Calendar) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO calendars (id, account_id, external_id, name, color, is_primary, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (account_id, external_id) DO UPDATE SET \
             name = EXCLUDED.name, color = EXCLUDED.color, \
             is_primary = EXCLUDED.is_primary, updated_at = EXCLUDED.updated_at",
        )
        .bind(calendar.id)
        .bind(calendar.account_id)
        .bind(&calendar.external_id)
        .bind(&calendar.name)
        .bind(&calendar.color)
        .bind(calendar.is_primary)
        .bind(calendar.created_at)
        .bind(calendar.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM calendars WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
