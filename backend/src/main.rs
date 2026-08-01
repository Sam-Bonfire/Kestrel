mod api;
mod config;
mod core;
mod db;
mod plugins;

use api::router::{AppState, create_router};
use api::rate_limit::RateLimiter;
use api::sync::{SyncEvent, start_sync_daemon};
use config::Config;
use core::offline_worker::start_offline_worker;
use db::pool::{init_pool, run_migrations};
use plugins::manager::PluginManager;
use plugins::mock::MockProviderPlugin;
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

    use plugins::gmail::GmailProviderPlugin;
    use plugins::outlook::OutlookProviderPlugin;

    // Initialize plugin manager
    let mut plugin_manager = PluginManager::new();

    if let (Some(client_id), Some(client_secret)) = (config.gmail_client_id, config.gmail_client_secret) {
        plugin_manager.register(Box::new(GmailProviderPlugin::new(client_id, client_secret)));
    } else {
        tracing::warn!("GMAIL_CLIENT_ID or GMAIL_CLIENT_SECRET not set, falling back to MockProviderPlugin for Gmail");
        plugin_manager.register(Box::new(MockProviderPlugin::new("gmail", "Gmail")));
    }

    if let (Some(client_id), Some(client_secret)) = (config.outlook_client_id, config.outlook_client_secret) {
        plugin_manager.register(Box::new(OutlookProviderPlugin::new(client_id, client_secret)));
    } else {
        tracing::warn!("OUTLOOK_CLIENT_ID or OUTLOOK_CLIENT_SECRET not set, falling back to MockProviderPlugin for Outlook");
        plugin_manager.register(Box::new(MockProviderPlugin::new("outlook", "Outlook")));
    }

    // Attempt to load WASM plugins from the plugins directory
    if let Err(e) = plugin_manager.load_all().await {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    let plugin_manager = Arc::new(tokio::sync::RwLock::new(plugin_manager));

    // Create broadcast channel for sync events
    let (sync_tx, _) = broadcast::channel::<SyncEvent>(256);

    let state = AppState {
        db: db.clone(),
        jwt_secret: config.jwt_secret,
        plugin_manager,
        sync_tx: sync_tx.clone(),
        auth_rate_limiter: RateLimiter::new(10, std::time::Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(100, std::time::Duration::from_secs(60)),
    };

    // Start background sync daemon
    start_sync_daemon(state.clone(), sync_tx);
    start_offline_worker(db.clone());

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
