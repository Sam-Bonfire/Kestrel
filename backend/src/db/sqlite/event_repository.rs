use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::CalendarEvent;
use crate::core::repository::EventRepository;

pub struct SqliteEventRepository {
    pool: SqlitePool,
}

impl SqliteEventRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

const EVENT_COLUMNS: &str = "e.id, e.account_id, e.calendar_id, e.external_id, e.title, e.description, e.location, \
     e.start_time, e.end_time, e.is_all_day, e.recurrence_rules, e.organizer_email, \
     e.organizer_name, e.attendees, e.status, e.has_conflict, e.created_at, e.updated_at";

#[async_trait]
impl EventRepository for SqliteEventRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(&format!(
            "SELECT {EVENT_COLUMNS} FROM calendar_events e WHERE e.id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(&format!(
            "SELECT {EVENT_COLUMNS} FROM calendar_events e WHERE e.account_id = ? AND e.external_id = ?"
        ))
        .bind(account_id.to_string())
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
                 WHERE a.user_id = ? AND e.end_time >= ? AND e.start_time <= ? \
                 AND e.calendar_id = ? \
                 ORDER BY e.start_time"
            ))
            .bind(user_id.to_string())
            .bind(start_time)
            .bind(end_time)
            .bind(calendar_id.to_string())
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CalendarEvent>(&format!(
                "SELECT {EVENT_COLUMNS} \
                 FROM calendar_events e \
                 JOIN accounts a ON e.account_id = a.id \
                 WHERE a.user_id = ? AND e.end_time >= ? AND e.start_time <= ? \
                 ORDER BY e.start_time"
            ))
            .bind(user_id.to_string())
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.pool)
            .await
        }
    }

    async fn upsert(&self, event: &CalendarEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO calendar_events (id, account_id, calendar_id, external_id, title, description, \
             location, start_time, end_time, is_all_day, recurrence_rules, organizer_email, \
             organizer_name, attendees, status, has_conflict, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(account_id, external_id) DO UPDATE SET \
             calendar_id = excluded.calendar_id, title = excluded.title, \
             description = excluded.description, location = excluded.location, \
             start_time = excluded.start_time, end_time = excluded.end_time, \
             is_all_day = excluded.is_all_day, recurrence_rules = excluded.recurrence_rules, \
             organizer_email = excluded.organizer_email, organizer_name = excluded.organizer_name, \
             attendees = excluded.attendees, status = excluded.status, has_conflict = excluded.has_conflict, \
             updated_at = excluded.updated_at",
        )
        .bind(event.id.to_string())
        .bind(event.account_id.to_string())
        .bind(event.calendar_id.to_string())
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
             updated_at = unixepoch() WHERE id = ?",
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
