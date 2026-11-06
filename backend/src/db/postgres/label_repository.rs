use async_trait::async_trait;
use uuid::Uuid;

use crate::core::models::Label;
use crate::core::repository::LabelRepository;

pub struct PostgresLabelRepository {
    pool: sqlx::PgPool,
}

impl PostgresLabelRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LabelRepository for PostgresLabelRepository {
    async fn update(&self, label: &Label) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE labels SET
                name = $1,
                color = $2,
                icon = $3,
                updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(&label.name)
        .bind(&label.color)
        .bind(&label.icon)
        .bind(label.updated_at)
        .bind(label.id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Label>, sqlx::Error> {
        sqlx::query_as::<_, Label>(
            "SELECT id, account_id, name, color, icon, created_at, updated_at FROM labels WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_by_accounts(&self, account_ids: &[Uuid]) -> Result<Vec<Label>, sqlx::Error> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<_, Label>(
            "SELECT id, account_id, name, color, icon, created_at, updated_at \
             FROM labels WHERE account_id = ANY($1) ORDER BY name",
        )
        .bind(account_ids)
        .fetch_all(&self.pool)
        .await
    }
}
