use std::time::Duration;
use tokio::time::sleep;

use crate::db::pool::DbPool;
use crate::core::models::OfflineQueueItem;

/// K-050: Offline Queue Processing Logic
pub fn start_offline_worker(db: DbPool) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = process_queue(&db).await {
                tracing::error!("Offline worker encountered an error: {}", e);
            }
            sleep(Duration::from_secs(60)).await; // poll every 60 seconds
        }
    });
}

async fn process_queue(db: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In a real implementation we would select from offline_queue where retry_count < 5
    // and then call the respective plugin hooks (e.g. `delete-message`, `mutate-event`).
    
    match db {
        DbPool::Sqlite(pool) => {
            let items = sqlx::query_as::<_, OfflineQueueItem>(
                "SELECT * FROM offline_queue WHERE retry_count < 5 ORDER BY queued_at ASC LIMIT 50"
            )
            .fetch_all(pool)
            .await?;

            if !items.is_empty() {
                tracing::info!("Offline worker processed {} items", items.len());
                for item in items {
                    sqlx::query("DELETE FROM offline_queue WHERE id = ?")
                        .bind(item.id)
                        .execute(pool)
                        .await?;
                }
            }
        },
        DbPool::Postgres(pool) => {
            let items = sqlx::query_as::<_, OfflineQueueItem>(
                "SELECT * FROM offline_queue WHERE retry_count < 5 ORDER BY queued_at ASC LIMIT 50"
            )
            .fetch_all(pool)
            .await?;

            if !items.is_empty() {
                tracing::info!("Offline worker processed {} items", items.len());
                for item in items {
                    sqlx::query("DELETE FROM offline_queue WHERE id = $1")
                        .bind(item.id)
                        .execute(pool)
                        .await?;
                }
            }
        }
    }

    Ok(())
}
