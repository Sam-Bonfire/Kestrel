use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::CalendarEvent;
use crate::core::repository::EventRepository;

pub struct PostgresEventRepository {
    pool: PgPool,
}

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const EVENT_COLUMNS: &str = "e.id, e.account_id, e.calendar_id, e.external_id, e.title, e.description, e.location, \
     e.start_time, e.end_time, e.is_all_day, e.recurrence_rules, e.organizer_email, \
     e.organizer_name, e.attendees::TEXT as attendees, e.status, e.created_at, e.updated_at";

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(&format!(
            "SELECT {EVENT_COLUMNS} FROM calendar_events e WHERE e.id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(&format!(
            "SELECT {EVENT_COLUMNS} FROM calendar_events e WHERE e.account_id = $1 AND e.external_id = $2"
        ))
        .bind(account_id)
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_range(
        &self,
        user_id: Uuid,
        start_time: i64,
        end_time: i64,
        calendar_id: Option<Uuid>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        if let Some(calendar_id) = calendar_id {
            sqlx::query_as::<_, CalendarEvent>(&format!(
                "SELECT {EVENT_COLUMNS} \
                 FROM calendar_events e \
                 JOIN accounts a ON e.account_id = a.id \
                 WHERE a.user_id = $1 AND e.end_time >= $2 AND e.start_time <= $3 \
                 AND e.calendar_id = $4 \
                 ORDER BY e.start_time"
            ))
            .bind(user_id)
            .bind(start_time)
            .bind(end_time)
            .bind(calendar_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CalendarEvent>(&format!(
                "SELECT {EVENT_COLUMNS} \
                 FROM calendar_events e \
                 JOIN accounts a ON e.account_id = a.id \
                 WHERE a.user_id = $1 AND e.end_time >= $2 AND e.start_time <= $3 \
                 ORDER BY e.start_time"
            ))
            .bind(user_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.pool)
            .await
        }
    }

    async fn search(
        &self,
        user_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(&format!(
            "SELECT {EVENT_COLUMNS} \
             FROM calendar_events e \
             JOIN accounts a ON e.account_id = a.id \
             WHERE a.user_id = $1 \
             AND (e.title ILIKE '%' || $2 || '%' \
                  OR e.description ILIKE '%' || $2 || '%' \
                  OR e.location ILIKE '%' || $2 || '%') \
             ORDER BY e.start_time LIMIT $3"
        ))
        .bind(user_id)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert(&self, event: &CalendarEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO calendar_events (id, account_id, calendar_id, external_id, title, description, \
             location, start_time, end_time, is_all_day, recurrence_rules, organizer_email, \
             organizer_name, attendees, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) \
             ON CONFLICT (account_id, external_id) DO UPDATE SET \
             calendar_id = EXCLUDED.calendar_id, title = EXCLUDED.title, \
             description = EXCLUDED.description, location = EXCLUDED.location, \
             start_time = EXCLUDED.start_time, end_time = EXCLUDED.end_time, \
             is_all_day = EXCLUDED.is_all_day, recurrence_rules = EXCLUDED.recurrence_rules, \
             organizer_email = EXCLUDED.organizer_email, organizer_name = EXCLUDED.organizer_name, \
             attendees = EXCLUDED.attendees, status = EXCLUDED.status, \
             updated_at = EXCLUDED.updated_at",
        )
        .bind(event.id)
        .bind(event.account_id)
        .bind(event.calendar_id)
        .bind(&event.external_id)
        .bind(&event.title)
        .bind(&event.description)
        .bind(&event.location)
        .bind(event.start_time)
        .bind(event.end_time)
        .bind(event.is_all_day)
        .bind(&event.recurrence_rules)
        .bind(&event.organizer_email)
        .bind(&event.organizer_name)
        .bind(&event.attendees)
        .bind(&event.status)
        .bind(event.created_at)
        .bind(event.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE calendar_events SET title = '[deleted]', description = NULL, location = NULL, \
             updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM calendar_events WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
