mod config;
mod downloader;
mod events;
mod extractor;
mod handlers;
mod initialization;
mod launch;
mod server;
mod utils;

use crate::utils::paths;
use tokio::sync::oneshot;
use tracing::{Level, error, info};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with a more verbose default level
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stdout)
        .init();

    info!("Logging initialized at DEBUG level");

    // Load application configuration
    let app_config = config::AppConfig::from_env();

    // Run initialization checks and setup from the old run.sh
    if let Err(e) = initialization::initialize(&app_config) {
        error!("Initialization failed: {}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ));
    }

    // Check if we should directly launch Foundry
    if paths::FOUNDRY_SCRIPT_PATH.exists() {
        info!("Foundry main.js detected, skipping Actix server and launching Foundry directly");
        launch::launch_foundry_process(None, &app_config).await;
        return Ok(());
    }

    // Log configuration settings
    info!("Serving static files from: {}", app_config.static_files_dir);
    info!("Downloading files to: {}", app_config.target_dir);

    // Create a channel for shutting down Foundry when needed
    let (_foundry_tx, foundry_rx) = oneshot::channel::<()>();

    // Start the HTTP server
    let server_handle = server::start_server(&app_config).await?;

    // Wait for the server to complete (after receiving shutdown signal)
    // Fix: Explicitly acknowledge the Result with let _
    let _ = server_handle.await?;
    info!("Actix server has terminated, launching Foundry VTT");

    // After server stops, launch Foundry directly with the shutdown channel
    launch::launch_foundry_process(Some(foundry_rx), &app_config).await;

    Ok(())
}
