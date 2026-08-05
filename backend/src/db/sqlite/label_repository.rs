use async_trait::async_trait;
use uuid::Uuid;

use crate::core::models::Label;
use crate::core::repository::LabelRepository;

pub struct SqliteLabelRepository {
    pool: sqlx::SqlitePool,
}

impl SqliteLabelRepository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LabelRepository for SqliteLabelRepository {
    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<Label>, sqlx::Error> {
        let account_id_str = account_id.to_string();
        sqlx::query_as::<_, Label>(
            r#"
            SELECT * FROM labels WHERE account_id = ?
            "#,
        )
        .bind(account_id_str)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert(&self, label: &Label) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO labels (id, account_id, name, color, icon, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, name) DO UPDATE SET
                color = excluded.color,
                icon = excluded.icon,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(label.id.to_string())
        .bind(label.account_id.to_string())
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
        let id_str = id.to_string();
        sqlx::query("DELETE FROM labels WHERE id = ?")
            .bind(id_str)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
