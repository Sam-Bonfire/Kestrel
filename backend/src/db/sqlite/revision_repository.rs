use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::HistoricalRevision;
use crate::core::repository::HistoricalRevisionRepository;

#[allow(dead_code)]
pub struct SqliteRevisionRepository {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl SqliteRevisionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HistoricalRevisionRepository for SqliteRevisionRepository {
    async fn create(&self, revision: &HistoricalRevision) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO historical_revisions (id, resource_type, resource_id, serialized_payload, revision_number, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(revision.id.to_string())
        .bind(&revision.resource_type)
        .bind(revision.resource_id.to_string())
        .bind(&revision.serialized_payload)
        .bind(revision.revision_number)
        .bind(revision.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<HistoricalRevision>, sqlx::Error> {
        sqlx::query_as::<_, HistoricalRevision>(
            "SELECT id, resource_type, resource_id, serialized_payload, revision_number, created_at \
             FROM historical_revisions WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_latest_revision_number(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i32>(
            "SELECT COALESCE(MAX(revision_number), 0) FROM historical_revisions \
             WHERE resource_type = ? AND resource_id = ?",
        )
        .bind(resource_type)
        .bind(resource_id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(result)
    }
}
