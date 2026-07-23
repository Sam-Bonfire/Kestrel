use std::path::Path;

use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Postgres, Sqlite};
use tracing::info;

#[derive(Clone)]
pub enum DbPool {
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<Postgres>),
}

pub async fn init_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        info!("Connected to PostgreSQL database");
        Ok(DbPool::Postgres(pool))
    } else if database_url.starts_with("sqlite:") {
        let path_str = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
        let path = Path::new(path_str);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = SqliteConnectOptions::new()
            .filename(path_str)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;

        info!("Connected to SQLite database at {}", path_str);
        Ok(DbPool::Sqlite(pool))
    } else {
        Err(format!("Unsupported DATABASE_URL scheme: {}", database_url).into())
    }
}

pub async fn run_migrations(db: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    match db {
        DbPool::Sqlite(pool) => {
            let migration_path = format!("{}/migrations/sqlite/001_initial.sql", manifest_dir);
            let migration_sql = std::fs::read_to_string(&migration_path)?;
            sqlx::raw_sql(&migration_sql).execute(pool).await?;
            info!("SQLite migrations completed");
        }
        DbPool::Postgres(pool) => {
            let migration_path = format!("{}/migrations/postgres/001_initial.sql", manifest_dir);
            let migration_sql = std::fs::read_to_string(&migration_path)?;
            sqlx::raw_sql(&migration_sql).execute(pool).await?;
            info!("PostgreSQL migrations completed");
        }
    }
    Ok(())
}
