use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::HistoricalRevision;
use crate::core::repository::HistoricalRevisionRepository;

#[allow(dead_code)]
pub struct PostgresRevisionRepository {
    pool: PgPool,
}

#[allow(dead_code)]
impl PostgresRevisionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HistoricalRevisionRepository for PostgresRevisionRepository {
    async fn create(&self, revision: &HistoricalRevision) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO historical_revisions (id, resource_type, resource_id, serialized_payload, revision_number, created_at) \
             VALUES ($1, $2, $3, $4::jsonb, $5, $6)"
        )
        .bind(revision.id)
        .bind(&revision.resource_type)
        .bind(revision.resource_id)
        .bind(&revision.serialized_payload)
        .bind(revision.revision_number)
        .bind(revision.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<HistoricalRevision>, sqlx::Error> {
        sqlx::query_as::<_, HistoricalRevision>(
            "SELECT id, resource_type, resource_id, serialized_payload::text, revision_number, created_at \
             FROM historical_revisions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn get_latest_revision_number(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(revision_number) FROM historical_revisions \
             WHERE resource_type = $1 AND resource_id = $2",
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.unwrap_or(0))
    }
}
