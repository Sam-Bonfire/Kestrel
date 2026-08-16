use async_trait::async_trait;

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
}
