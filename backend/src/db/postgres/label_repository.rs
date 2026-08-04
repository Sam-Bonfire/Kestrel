use async_trait::async_trait;
use uuid::Uuid;

use crate::core::models::Label;
use crate::core::repository::LabelRepository;
use crate::core::types::DbUuid;

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
    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Label>, sqlx::Error> {
        let account_id_db = DbUuid(account_id);
        sqlx::query_as::<_, Label>(
            r#"
            SELECT * FROM labels WHERE account_id = $1
            "#,
        )
        .bind(account_id_db)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert(&self, label: &Label) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO labels (id, account_id, name, color, icon, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(account_id, name) DO UPDATE SET
                color = excluded.color,
                icon = excluded.icon,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(label.id)
        .bind(label.account_id)
        .bind(&label.name)
        .bind(&label.color)
        .bind(&label.icon)
        .bind(label.created_at)
        .bind(label.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        let id_db = DbUuid(id);
        sqlx::query("DELETE FROM labels WHERE id = $1")
            .bind(id_db)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
