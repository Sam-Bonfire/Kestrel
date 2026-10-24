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

    async fn list_by_accounts(&self, account_ids: &[Uuid]) -> Result<Vec<Label>, sqlx::Error> {
        if account_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = vec!["?"; account_ids.len()].join(", ");
        let query = format!(
            "SELECT id, account_id, name, color, icon, created_at, updated_at \
             FROM labels WHERE account_id IN ({placeholders}) ORDER BY name"
        );
        // Audited: placeholders are generated from the account count only;
        // every value goes through a bind.
        let mut q = sqlx::query_as::<_, Label>(sqlx::AssertSqlSafe(query));
        for id in account_ids {
            q = q.bind(id.to_string());
        }
        q.fetch_all(&self.pool).await
    }
}
