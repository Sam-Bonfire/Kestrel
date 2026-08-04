use crate::api::router::AppState;
use crate::db::pool::DbPool;
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;

pub fn start_token_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Starting background Token Refresh Worker");

        loop {
            // Run every 5 minutes
            sleep(Duration::from_secs(300)).await;

            // We want to find accounts whose token expires in less than 10 minutes (600 seconds)
            let threshold = Utc::now().timestamp() + 600;

            // Let's implement the actual logic directly with sqlx since we need all accounts globally
            let mut expiring_accounts = match &state.db {
                DbPool::Sqlite(pool) => {
                    sqlx::query_as::<_, crate::core::models::Account>(
                        "SELECT * FROM accounts WHERE token_expires_at IS NOT NULL AND token_expires_at < ? AND refresh_token IS NOT NULL"
                    )
                    .bind(threshold)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default()
                },
                DbPool::Postgres(pool) => {
                    sqlx::query_as::<_, crate::core::models::Account>(
                        "SELECT * FROM accounts WHERE token_expires_at IS NOT NULL AND token_expires_at < $1 AND refresh_token IS NOT NULL"
                    )
                    .bind(threshold)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default()
                }
            };

            if expiring_accounts.is_empty() {
                continue;
            }

            tracing::info!(
                "Found {} accounts requiring token refresh",
                expiring_accounts.len()
            );

            let client = reqwest::Client::new();

            for mut account in expiring_accounts {
                let (token_url, client_id, client_secret) = match account.provider.as_str() {
                    "gmail" => (
                        "https://oauth2.googleapis.com/token",
                        std::env::var("GMAIL_CLIENT_ID").unwrap_or_default(),
                        std::env::var("GMAIL_CLIENT_SECRET").unwrap_or_default(),
                    ),
                    "outlook" => (
                        "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                        std::env::var("OUTLOOK_CLIENT_ID").unwrap_or_default(),
                        std::env::var("OUTLOOK_CLIENT_SECRET").unwrap_or_default(),
                    ),
                    _ => continue,
                };

                if client_id.is_empty() || client_secret.is_empty() {
                    continue;
                }

                let refresh_token = account.refresh_token.as_ref().unwrap();

                let res = client
                    .post(token_url)
                    .form(&[
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("refresh_token", refresh_token.as_str()),
                        ("grant_type", "refresh_token"),
                    ])
                    .send()
                    .await;

                match res {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(token_data) = resp.json::<serde_json::Value>().await {
                            if let Some(access_token) = token_data["access_token"].as_str() {
                                account.access_token = Some(access_token.to_string());
                                let expires_in = token_data["expires_in"].as_i64().unwrap_or(3600);
                                account.token_expires_at =
                                    Some(Utc::now().timestamp() + expires_in);
                                account.updated_at = Utc::now().timestamp();

                                // Save back to database
                                match &state.db {
                                    DbPool::Sqlite(pool) => {
                                        let update_res = sqlx::query("UPDATE accounts SET access_token = ?, token_expires_at = ?, updated_at = ? WHERE id = ?")
                                            .bind(&account.access_token)
                                            .bind(account.token_expires_at)
                                            .bind(account.updated_at)
                                            .bind(account.id)
                                            .execute(pool)
                                            .await;
                                        if let Err(e) = update_res {
                                            tracing::error!(
                                                "Failed to update refreshed token for account {}: {}",
                                                account.id.0,
                                                e
                                            );
                                        } else {
                                            tracing::info!(
                                                "Successfully refreshed token for account {}",
                                                account.id.0
                                            );
                                        }
                                    }
                                    DbPool::Postgres(pool) => {
                                        let update_res = sqlx::query("UPDATE accounts SET access_token = $1, token_expires_at = $2, updated_at = $3 WHERE id = $4")
                                            .bind(&account.access_token)
                                            .bind(account.token_expires_at)
                                            .bind(account.updated_at)
                                            .bind(account.id)
                                            .execute(pool)
                                            .await;
                                        if let Err(e) = update_res {
                                            tracing::error!(
                                                "Failed to update refreshed token for account {}: {}",
                                                account.id.0,
                                                e
                                            );
                                        } else {
                                            tracing::info!(
                                                "Successfully refreshed token for account {}",
                                                account.id.0
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(resp) => {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        tracing::error!(
                            "Token refresh failed for account {} ({}): {}",
                            account.id.0,
                            status,
                            body
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Network error refreshing token for account {}: {}",
                            account.id.0,
                            e
                        );
                    }
                }
            }
        }
    });
}
