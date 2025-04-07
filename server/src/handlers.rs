use crate::downloader::DownloadService;
use crate::extractor::ExtractorService;
use crate::server::AppState;
use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, warn};

#[derive(Deserialize)]
pub struct UrlPayload {
    url: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub async fn download_and_extract(
    url_payload: web::Json<UrlPayload>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let url = url_payload.url.clone();
    let target_directory = crate::config::get_target_directory();
    info!("Received request to download and extract from URL: {}", url);
    debug!("Target directory for extraction: {}", target_directory);

    // Ensure target directory exists
    if !Path::new(&target_directory).exists() {
        if let Err(e) = fs::create_dir_all(&target_directory).await {
            error!("Failed to create target directory: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create target directory: {}", e),
            });
        }
        info!("Created target directory: {}", target_directory);
    }

    let archive_path = format!("{}/archive.zip", target_directory);
    debug!("Archive will be saved to: {}", archive_path);

    // Download the archive
    if let Err(e) = DownloadService::download_file_from_url(&url, &archive_path).await {
        error!("Download error: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to download file: {}", e),
        });
    }

    // Verify file exists and has content before extraction
    match fs::metadata(&archive_path).await {
        Ok(metadata) => {
            let size_bytes = metadata.len();
            if size_bytes == 0 {
                error!("Downloaded file is empty (0 bytes)");
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Downloaded file is empty".to_string(),
                });
            }
            info!("Downloaded file size: {} bytes", size_bytes);
        }
        Err(e) => {
            error!("Failed to check downloaded file: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to verify downloaded file: {}", e),
            });
        }
    }

    // Extract the archive
    if let Err(e) =
        ExtractorService::extract_zip(archive_path.clone(), target_directory.clone()).await
    {
        error!("Extraction error: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to extract file: {}", e),
        });
    }

    // Cleanup just the zip file, not other content
    if let Err(e) = fs::remove_file(&archive_path).await {
        error!("Failed to delete temporary zip file: {}", e);
        // Continue despite cleanup failure
    } else {
        info!("Deleted temporary zip file: {}", archive_path);
    }

    info!(
        "Successfully downloaded and extracted content from: {}",
        url
    );

    // Check for foundry script existence
    let foundry_script_path = Path::new("/foundryvtt/resources/app/main.js");
    if foundry_script_path.exists() {
        info!(
            "Foundry main.js detected after extraction: {}",
            foundry_script_path.display()
        );
    } else {
        warn!(
            "Foundry main.js not found at expected path after extraction: {}",
            foundry_script_path.display()
        );
    }

    // Signal the server to shutdown
    if let Some(tx) = app_state.shutdown_sender.lock().unwrap().take() {
        let _ = tx.send(());
        info!("Sent shutdown signal to Actix server");
    } else {
        warn!("Shutdown channel was already used or unavailable");
    }

    HttpResponse::Ok().json(SuccessResponse {
        message: format!("Downloaded and extracted content from: {}", url),
    })
}
