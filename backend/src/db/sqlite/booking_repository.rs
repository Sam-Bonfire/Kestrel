use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::{BookingPage, CalendarEvent};
use crate::core::repository::BookingPageRepository;

pub struct SqliteBookingRepository {
    pool: SqlitePool,
}

impl SqliteBookingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

const COLUMNS: &str = "id, user_id, calendar_id, name, slug, duration_mins, \
     buffer_mins, window_days, is_active, created_at, updated_at";

#[async_trait]
impl BookingPageRepository for SqliteBookingRepository {
    async fn create(&self, page: &BookingPage) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO booking_pages (id, user_id, calendar_id, name, slug, \
             duration_mins, buffer_mins, window_days, is_active, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(page.id.to_string())
        .bind(page.user_id.to_string())
        .bind(page.calendar_id.to_string())
        .bind(&page.name)
        .bind(&page.slug)
        .bind(page.duration_mins)
        .bind(page.buffer_mins)
        .bind(page.window_days)
        .bind(page.is_active)
        .bind(page.created_at)
        .bind(page.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<BookingPage>, sqlx::Error> {
        sqlx::query_as::<_, BookingPage>(&format!(
            "SELECT {COLUMNS} FROM booking_pages WHERE user_id = ? ORDER BY created_at"
        ))
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<BookingPage>, sqlx::Error> {
        sqlx::query_as::<_, BookingPage>(&format!(
            "SELECT {COLUMNS} FROM booking_pages WHERE slug = ?"
        ))
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BookingPage>, sqlx::Error> {
        sqlx::query_as::<_, BookingPage>(&format!(
            "SELECT {COLUMNS} FROM booking_pages WHERE id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
    }

    async fn update(&self, page: &BookingPage) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE booking_pages SET calendar_id = ?, name = ?, slug = ?, \
             duration_mins = ?, buffer_mins = ?, window_days = ?, is_active = ?, \
             updated_at = ? WHERE id = ?",
        )
        .bind(page.calendar_id.to_string())
        .bind(&page.name)
        .bind(&page.slug)
        .bind(page.duration_mins)
        .bind(page.buffer_mins)
        .bind(page.window_days)
        .bind(page.is_active)
        .bind(page.updated_at)
        .bind(page.id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM booking_pages WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn insert_event_if_free(
        &self,
        event: &CalendarEvent,
        pad_start: i64,
        pad_end: i64,
    ) -> Result<bool, sqlx::Error> {
        // Single statement: the NOT EXISTS gate and the insert execute
        // atomically, so two concurrent bookings cannot both succeed.
        let result = sqlx::query(
            "INSERT INTO calendar_events (id, account_id, calendar_id, external_id, title, \
             description, location, start_time, end_time, is_all_day, recurrence_rules, \
             organizer_email, organizer_name, attendees, status, has_conflict, \
             created_at, updated_at) \
             SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? \
             WHERE NOT EXISTS ( \
               SELECT 1 FROM calendar_events \
               WHERE calendar_id = ? AND end_time > ? AND start_time < ? \
             )",
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
        .bind(event.has_conflict)
        .bind(event.created_at)
        .bind(event.updated_at)
        .bind(event.calendar_id.to_string())
        .bind(pad_start)
        .bind(pad_end)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }
}
