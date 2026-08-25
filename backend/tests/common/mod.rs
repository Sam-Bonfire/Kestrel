use backend::db::pool::{DbPool, run_migrations};
use sqlx::SqlitePool;

pub async fn setup_test_db() -> DbPool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory SQLite database");

    let db_pool = DbPool::Sqlite(pool.clone());
    run_migrations(&db_pool)
        .await
        .expect("Failed to run migrations for test DB");
    let _ = sqlx::query("ALTER TABLE messages ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;")
        .execute(&pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE calendar_events ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;",
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS historical_revisions (id TEXT PRIMARY KEY NOT NULL, resource_type TEXT NOT NULL, resource_id TEXT NOT NULL, serialized_payload TEXT NOT NULL, revision_number INTEGER NOT NULL, created_at INTEGER NOT NULL DEFAULT (unixepoch()));").execute(&pool).await;
    return db_pool;
}
