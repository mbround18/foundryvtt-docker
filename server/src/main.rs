// main.rs
mod launch;

use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::oneshot;
use tokio::task;
use tracing::{Level, debug, error, info, warn};
use tracing_actix_web::TracingLogger;
use zip::read::ZipArchive;

#[derive(Deserialize)]
struct UrlPayload {
    url: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    message: String,
}

// ---------- Helpers ----------

fn get_target_directory() -> String {
    // Check for TARGET_DIR first, then APPLICATION_DIR, then fallback
    env::var("TARGET_DIR").unwrap_or_else(|_| {
        env::var("APPLICATION_DIR").unwrap_or_else(|_| {
            let mut dir = env::current_dir().expect("Failed to get current directory");
            dir.push("tmp");
            // Make sure the directory exists
            std::fs::create_dir_all(&dir).expect("Failed to create target directory");
            dir.to_str().unwrap().to_string()
        })
    })
}

/// Download file from `url` and write it to `save_path`, streaming the response.
async fn download_file_from_url(url: &str, save_path: &str) -> Result<(), actix_web::Error> {
    info!("Starting download from URL: {}", url);
    let mut resp = Client::new().get(url).send().await.map_err(|e| {
        error!("Request error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to send request")
    })?;

    let mut out = fs::File::create(save_path)
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create file"))?;
    info!("Saving downloaded file to: {}", save_path);

    while let Some(chunk) = resp.chunk().await.map_err(|_| {
        error!("Failed reading download stream");
        actix_web::error::ErrorInternalServerError("Failed reading download stream")
    })? {
        use tokio::io::AsyncWriteExt;
        out.write_all(&chunk)
            .await
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to write file"))?;
    }

    info!("Download completed successfully");
    Ok(())
}

/// Extract ZIP at `archive_path` into `target_directory` using a blocking task.
async fn extract_zip(archive_path: String, target_directory: String) -> Result<(), std::io::Error> {
    info!("Starting extraction of archive: {}", archive_path);
    task::spawn_blocking(move || {
        let file = File::open(&archive_path)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(&target_directory)?;
        Ok(())
    })
    .await
    .unwrap_or_else(|e| {
        error!("Blocking task panicked: {}", e);
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Extraction thread panicked",
        ))
    })?;
    info!("Extraction completed successfully");
    Ok(())
}

// ---------- Handlers ----------

async fn download_and_extract(
    url_payload: web::Json<UrlPayload>,
    server_shutdown: web::Data<Arc<Mutex<Option<oneshot::Sender<()>>>>>,
) -> impl Responder {
    let url = url_payload.url.clone();
    let target_directory = get_target_directory();
    info!("Received request to download and extract from URL: {}", url);

    // Ensure target directory exists
    if !Path::new(&target_directory).exists() {
        if let Err(e) = fs::create_dir_all(&target_directory).await {
            error!("Failed to create target directory: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
        info!("Created target directory: {}", target_directory);
    }

    let archive_path = format!("{}/archive.zip", target_directory);

    // Download the archive
    if let Err(e) = download_file_from_url(&url, &archive_path).await {
        error!("Download error: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // Extract the archive
    if let Err(e) = extract_zip(archive_path.clone(), target_directory.clone()).await {
        error!("Extraction error: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // Cleanup just the zip file, not other content
    if let Err(e) = fs::remove_file(&archive_path).await {
        error!("Failed to delete temporary zip file: {}", e);
    } else {
        info!("Deleted temporary zip file: {}", archive_path);
    }

    info!(
        "Successfully downloaded and extracted content from: {}",
        url
    );

    // Signal the server to shutdown
    if let Some(tx) = server_shutdown.lock().unwrap().take() {
        let _ = tx.send(());
        info!("Sent shutdown signal to Actix server");
    } else {
        warn!("Shutdown channel was already used or unavailable");
    }

    HttpResponse::Ok().json(SuccessResponse {
        message: format!("Downloaded and extracted content from: {}", url),
    })
}

async fn launch_foundry_process(shutdown_rx: Option<oneshot::Receiver<()>>) {
    let foundry_args = vec![
        "--dataPath=/foundrydata",
        "--port=4444",
        "--hostname=foundry.vtt",
        "--noupnp",
        "--proxySSL",
    ];

    let foundry_script = "/foundryvtt/resources/app/main.js".to_string();

    info!("Starting Foundry VTT on port 4444");

    // Launch Foundry in the same task, passing the shutdown channel
    launch::launch_foundry(&foundry_args, &foundry_script, shutdown_rx).await;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with a more verbose default level
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stdout)
        .init();

    info!("Logging initialized at DEBUG level");

    // Check multiple environment variables with priority
    let static_files_dir = env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "static".to_string());
    let server_port = env::var("SERVER_PORT")
        .or_else(|_| env::var("APPLICATION_PORT"))
        .unwrap_or_else(|_| "4444".to_string())
        .parse::<u16>()
        .unwrap_or(4444);
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let foundry_script_path = Path::new("/foundryvtt/resources/app/main.js");
    if foundry_script_path.exists() {
        info!("Foundry main.js detected, skipping Actix server and launching Foundry directly");
        launch_foundry_process(None).await;
        return Ok(());
    }


    info!("Serving static files from: {}", static_files_dir);
    info!("Downloading files to: {}", get_target_directory());

    // Create a channel for shutting down the server
    let (tx, rx) = oneshot::channel::<()>();
    // Wrap the Sender in Arc<Mutex<Option<...>>> so it can be shared and taken
    let shared_tx = Arc::new(Mutex::new(Some(tx)));

    // Create a channel for shutting down the Foundry process
    let (foundry_tx, foundry_rx) = oneshot::channel::<()>();
    let foundry_shutdown = Arc::new(Mutex::new(Some(foundry_tx)));

    info!("Server is running on {}:{}", server_host, server_port);
    debug!("Debug logging is enabled");

    // Start the server
    let server = HttpServer::new(move || {
        let tx_clone = Arc::clone(&shared_tx);
        App::new()
            // Logging for Actix with more details
            .wrap(TracingLogger::default())
            // Store the shutdown channel
            .app_data(web::Data::new(tx_clone))
            // Serve the download endpoint and static files
            .route("/download", web::post().to(download_and_extract))
            .service(Files::new("/", &static_files_dir).index_file("index.html"))
    })
    .bind((server_host, server_port))?
    .run();

    let server_handle = server.handle();

    // Spawn a task to wait for the shutdown signal
    tokio::spawn(async move {
        // If we receive the shutdown signal, stop the server gracefully
        if rx.await.is_ok() {
            info!("Received shutdown signal, stopping Actix server");
            server_handle.stop(true).await;
            info!("Actix server stopped, transitioning to process management mode");
        }
    });

    // Setup signal handlers for SIGTERM and SIGINT
    let foundry_shutdown_clone2 = Arc::clone(&foundry_shutdown);
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        // Handle SIGTERM
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let foundry_shutdown_sigterm = Arc::clone(&foundry_shutdown_clone2);
        tokio::spawn(async move {
            sigterm.recv().await;
            info!("Received SIGTERM, initiating shutdown");

            // Signal Foundry to shut down if it's running
            if let Some(tx) = foundry_shutdown_sigterm.lock().unwrap().take() {
                let _ = tx.send(());
                info!("Sent shutdown signal to Foundry VTT process");
            }

            // Exit process
            std::process::exit(0);
        });

        // Handle SIGINT
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let foundry_shutdown_sigint = Arc::clone(&foundry_shutdown_clone2);
        tokio::spawn(async move {
            sigint.recv().await;
            info!("Received SIGINT, initiating shutdown");

            // Signal Foundry to shut down if it's running
            if let Some(tx) = foundry_shutdown_sigint.lock().unwrap().take() {
                let _ = tx.send(());
                info!("Sent shutdown signal to Foundry VTT process");
            }

            // Exit process
            std::process::exit(0);
        });
    }

    // Wait for the server to complete (either by normal termination or shutdown signal)
    server.await?;
    info!("Actix server has terminated, launching Foundry VTT");

    // After server stops, launch Foundry directly with the shutdown channel
    launch_foundry_process(Some(foundry_rx)).await;

    Ok(())
}
