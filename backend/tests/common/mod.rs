use backend::db::pool::{DbPool, run_migrations};
use sqlx::SqlitePool;

pub async fn setup_test_db() -> DbPool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory SQLite database");

    let db_pool = DbPool::Sqlite(pool);

    run_migrations(&db_pool)
        .await
        .expect("Failed to run migrations for test DB");

    db_pool
}
