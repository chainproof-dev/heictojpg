//! HEIC to JPG Converter - High Performance Web Application
//!
//! A production-grade, super-fast HEIC to JPG converter built in Rust.

mod config;
mod converter;
mod error;
mod handlers;
mod worker;
mod state;
mod router;

use crate::config::Config;
use crate::state::AppState;
use crate::worker::WorkerPool;
use crate::router::create_router;

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "heictojpg=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting HEIC to JPG converter...");

    // Load configuration
    let config = Arc::new(Config::from_env());
    info!("Configuration loaded: {:?}", config);

    // Create upload directory
    if let Err(e) = tokio::fs::create_dir_all(&config.upload_dir).await {
        eprintln!("Failed to create upload directory: {}", e);
        std::process::exit(1);
    }
    info!(dir = config.upload_dir, "Upload directory verified");

    // Create worker pool
    let worker_pool = WorkerPool::new(&config);
    info!(workers = config.worker_count, "Worker pool initialized");

    // Create shared app state
    let app_state = Arc::new(AppState { 
        worker_pool,
        config: config.clone(),
    });

    // Build router
    let app = create_router(app_state);

    // Bind and serve
    let port = config.server_port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    info!(port = port, "Server listening");

    // Graceful shutdown handler
    let shutdown_signal = async {
        let _ = tokio::signal::ctrl_c().await;
        info!("Shutting down...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .expect("Server error");
}
