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
    async fn update(&self, label: &Label) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE labels SET
                name = ?,
                color = ?,
                icon = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&label.name)
        .bind(&label.color)
        .bind(&label.icon)
        .bind(label.updated_at)
        .bind(label.id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Label>, sqlx::Error> {
        sqlx::query_as::<_, Label>(
            "SELECT id, account_id, name, color, icon, created_at, updated_at FROM labels WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
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
