use crate::core::models::HistoricalRevision;
use async_trait::async_trait;
use uuid::Uuid;

use super::models::{Account, Calendar, CalendarEvent, Message, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create(&self, user: &User) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, sqlx::Error>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>, sqlx::Error>;
    async fn find_by_provider_account_id(
        &self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, sqlx::Error>;

    async fn create(&self, account: &Account) -> Result<(), sqlx::Error>;
    async fn update_tokens_and_error(
        &self,
        id: Uuid,
        access_token: Option<&str>,
        expires_at: Option<i64>,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, sqlx::Error>;
    async fn find_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<Message>, sqlx::Error>;
    async fn list(
        &self,
        account_id: Option<Uuid>,
        folder: Option<&str>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error>;
    async fn search(
        &self,
        user_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error>;
    async fn upsert(&self, message: &Message) -> Result<(), sqlx::Error>;
    async fn set_read(&self, id: Uuid, is_read: bool) -> Result<(), sqlx::Error>;
    async fn set_archived(&self, id: Uuid, is_archived: bool) -> Result<(), sqlx::Error>;
    async fn set_deleted(&self, id: Uuid, is_deleted: bool) -> Result<(), sqlx::Error>;
    async fn set_labels(&self, id: Uuid, labels: Option<String>) -> Result<(), sqlx::Error>;

    async fn set_thread_muted(&self, thread_id: &str) -> Result<(), sqlx::Error>;
    async fn report_phishing(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn trash_by_sender(&self, user_id: Uuid, email: &str) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait FilterRepository: Send + Sync {
    async fn block_sender(&self, user_id: Uuid, email: &str) -> Result<(), sqlx::Error>;
    async fn get_blocked_senders(&self, user_id: Uuid) -> Result<Vec<String>, sqlx::Error>;
}

#[async_trait]
pub trait CalendarRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Calendar>, sqlx::Error>;
    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Calendar>, sqlx::Error>;
    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Calendar>, sqlx::Error>;
    async fn upsert(&self, calendar: &Calendar) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CalendarEvent>, sqlx::Error>;
    async fn find_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<CalendarEvent>, sqlx::Error>;
    async fn list_range(
        &self,
        user_id: Uuid,
        start_time: i64,
        end_time: i64,
        calendar_id: Option<Uuid>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error>;

    async fn upsert(&self, event: &CalendarEvent) -> Result<(), sqlx::Error>;
    async fn soft_delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait LabelRepository: Send + Sync {
    async fn upsert(&self, label: &crate::core::models::Label) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait UserPreferencesRepository: Send + Sync {
    async fn get_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<Option<crate::core::models::UserPreferences>, sqlx::Error>;
    async fn update_preferences(
        &self,
        user_id: Uuid,
        preferences_json: &str,
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
pub trait ContactRepository: Send + Sync {
    async fn upsert(&self, contact: &crate::core::models::Contact) -> Result<(), sqlx::Error>;
    async fn search(
        &self,
        account_ids: &[Uuid],
        query: &str,
        limit: i64,
    ) -> Result<Vec<crate::core::models::Contact>, sqlx::Error>;
}

#[async_trait]
#[allow(dead_code)]
pub trait HistoricalRevisionRepository: Send + Sync {
    async fn create(&self, revision: &HistoricalRevision) -> Result<(), sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<HistoricalRevision>, sqlx::Error>;
    async fn get_latest_revision_number(&self, resource_type: &str, resource_id: Uuid) -> Result<i32, sqlx::Error>;
}
