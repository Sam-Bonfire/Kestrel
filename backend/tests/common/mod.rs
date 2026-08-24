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

    let _ = sqlx::query("ALTER TABLE messages ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE calendar_events ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;").execute(&pool).await;

    db_pool
}
