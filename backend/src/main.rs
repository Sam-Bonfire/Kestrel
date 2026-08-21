mod api;
mod config;
mod core;
mod db;
mod plugins;

use api::rate_limit::RateLimiter;
use api::router::{AppState, create_router};
use api::sync::{SyncEvent, start_sync_daemon};
use config::Config;
use core::offline_worker::start_offline_worker;
use db::pool::{init_pool, run_migrations};
use plugins::manager::PluginManager;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// K-061: Wait for Ctrl-C (SIGINT) to trigger graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down...");
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug".into()),
        )
        .init();

    let config = Config::from_env();

    let db = init_pool(&config.database_url).await?;
    run_migrations(&db).await?;

    // Initialize plugin manager
    let mut plugin_manager = PluginManager::new();

    // Attempt to load WASM plugins from the plugins directory
    if let Err(e) = plugin_manager.load_all().await {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    let plugin_manager = Arc::new(tokio::sync::RwLock::new(plugin_manager));

    // Create broadcast channel for sync events
    let (sync_tx, _) = broadcast::channel::<SyncEvent>(256);

    use crate::api::proxy::SafeDnsResolver;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(std::sync::Arc::new(SafeDnsResolver))
        .build()?;

    let state = AppState {
        db: db.clone(),
        jwt_secret: config.jwt_secret,
        plugin_manager,
        sync_tx: sync_tx.clone(),
        auth_rate_limiter: RateLimiter::new(10, std::time::Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(100, std::time::Duration::from_secs(60)),
        http_client,
    };

    // Start background sync daemon
    start_sync_daemon(state.clone(), sync_tx);
    start_offline_worker(db.clone());
    crate::api::token_worker::start_token_worker(state.clone());

    let router = create_router(state);

    tracing::info!("Starting Kestrel backend on {}", config.bind_addr);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;

    // K-061: Graceful shutdown on SIGINT / SIGTERM
    let server = axum::serve(listener, router);
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}
