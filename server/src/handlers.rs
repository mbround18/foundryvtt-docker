use crate::downloader::DownloadService;
use crate::events::ProgressEvent;
use crate::extractor::ExtractorService;
use crate::server::AppState;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder, web};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
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

pub async fn info() -> impl Responder {
    // This endpoint can be used to check the server status or provide information
    HttpResponse::Ok().json(SuccessResponse {
        message: "Server is running".to_string(),
    })
}

// Helper functions to reduce code duplication

/// Ensures the target directory exists and returns its path
async fn ensure_target_directory(
    event_tx: &broadcast::Sender<ProgressEvent>,
) -> Result<String, HttpResponse> {
    let target_directory = crate::config::get_target_directory();
    debug!("Target directory for extraction: {}", target_directory);

    // Ensure target directory exists
    if !Path::new(&target_directory).exists() {
        if let Err(e) = fs::create_dir_all(&target_directory).await {
            error!("Failed to create target directory: {}", e);
            let _ = event_tx.send(ProgressEvent::new(
                "error",
                &format!("Failed to create target directory: {}", e),
                None,
            ));
            return Err(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create target directory: {}", e),
            }));
        }
        info!("Created target directory: {}", target_directory);
    }

    Ok(target_directory)
}

/// Extracts a ZIP archive and cleans up the source file
async fn extract_and_cleanup(
    archive_path: String,
    target_directory: String,
    event_tx: broadcast::Sender<ProgressEvent>,
) -> Result<(), HttpResponse> {
    // Send extraction started event
    let _ = event_tx.send(ProgressEvent::new(
        "extracting",
        "Extracting archive...",
        Some(60.0),
    ));

    // Extract the archive
    if let Err(e) = ExtractorService::extract_zip(
        archive_path.clone(),
        target_directory.clone(),
        event_tx.clone(),
    )
    .await
    {
        error!("Extraction error: {}", e);
        let _ = event_tx.send(ProgressEvent::new(
            "error",
            &format!("Failed to extract file: {}", e),
            None,
        ));
        return Err(HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to extract file: {}", e),
        }));
    }

    // Cleanup just the zip file, not other content
    let _ = event_tx.send(ProgressEvent::new(
        "cleanup",
        "Cleaning up temporary files...",
        Some(90.0),
    ));

    if let Err(e) = fs::remove_file(&archive_path).await {
        error!("Failed to delete temporary zip file: {}", e);
        // Continue despite cleanup failure
    } else {
        info!("Deleted temporary zip file: {}", archive_path);
    }

    Ok(())
}

/// Completes the installation process by checking for Foundry and shutting down the installer
async fn complete_installation(
    event_tx: broadcast::Sender<ProgressEvent>,
    app_state: web::Data<AppState>,
    success_message: &str,
) -> HttpResponse {
    // Send completion event
    let _ = event_tx.send(ProgressEvent::new(
        "complete",
        "Download and extraction complete!",
        Some(100.0),
    ));

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

    // Signal the server to shut down
    if let Some(tx) = app_state.shutdown_sender.lock().unwrap().take() {
        let _ = tx.send(());
        info!("Sent shutdown signal to Actix server");
        let _ = event_tx.send(ProgressEvent::new(
            "transition",
            "Transitioning to Foundry VTT...",
            Some(100.0),
        ));
    } else {
        warn!("Shutdown channel was already used or unavailable");
    }

    HttpResponse::Ok().json(SuccessResponse {
        message: success_message.to_string(),
    })
}

pub async fn download_and_extract(
    url_payload: web::Json<UrlPayload>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let url = url_payload.url.clone();
    let event_tx = app_state.event_channel.clone();

    info!("Received request to download and extract from URL: {}", url);

    // Send initial progress event
    let _ = event_tx.send(ProgressEvent::new(
        "start",
        &format!("Starting download from {}", url),
        Some(0.0),
    ));

    // Ensure target directory exists
    let target_directory = match ensure_target_directory(&event_tx).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let archive_path = format!("{}/archive.zip", target_directory);
    debug!("Archive will be saved to: {}", archive_path);

    // Send download started event
    let _ = event_tx.send(ProgressEvent::new(
        "downloading",
        "Downloading archive...",
        Some(10.0),
    ));

    // Download the archive
    if let Err(e) =
        DownloadService::download_file_from_url(&url, &archive_path, event_tx.clone()).await
    {
        error!("Download error: {}", e);
        let _ = event_tx.send(ProgressEvent::new(
            "error",
            &format!("Failed to download file: {}", e),
            None,
        ));
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
                let _ = event_tx.send(ProgressEvent::new(
                    "error",
                    "Downloaded file is empty",
                    None,
                ));
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Downloaded file is empty".to_string(),
                });
            }
            info!("Downloaded file size: {} bytes", size_bytes);
            let _ = event_tx.send(ProgressEvent::new(
                "downloaded",
                &format!("Download complete: {} bytes", size_bytes),
                Some(50.0),
            ));
        }
        Err(e) => {
            error!("Failed to check downloaded file: {}", e);
            let _ = event_tx.send(ProgressEvent::new(
                "error",
                &format!("Failed to verify downloaded file: {}", e),
                None,
            ));
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to verify downloaded file: {}", e),
            });
        }
    }

    // Extract and cleanup
    if let Err(response) =
        extract_and_cleanup(archive_path, target_directory, event_tx.clone()).await
    {
        return response;
    }

    info!(
        "Successfully downloaded and extracted content from: {}",
        url
    );

    // Complete installation
    complete_installation(
        event_tx,
        app_state,
        &format!("Downloaded and extracted content from: {}", url),
    )
    .await
}

pub async fn upload_and_extract(
    mut payload: Multipart,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let event_tx = app_state.event_channel.clone();

    info!("Received file upload request");

    // Send initial progress event
    let _ = event_tx.send(ProgressEvent::new(
        "start",
        "Starting file upload process",
        Some(0.0),
    ));

    // Ensure target directory exists
    let target_directory = match ensure_target_directory(&event_tx).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let archive_path = format!("{}/archive.zip", target_directory);
    debug!("Archive will be saved to: {}", archive_path);

    // Process the uploaded file
    let mut file = match fs::File::create(&archive_path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create file: {}", e);
            let _ = event_tx.send(ProgressEvent::new(
                "error",
                &format!("Failed to create file: {}", e),
                None,
            ));
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create file: {}", e),
            });
        }
    };

    let mut total_bytes = 0;
    let mut field_name = String::new();

    // Process uploaded file chunks
    while let Some(field_result) = payload.next().await {
        let mut field = match field_result {
            Ok(field) => field,
            Err(e) => {
                error!("Error getting multipart field: {}", e);
                let _ = event_tx.send(ProgressEvent::new(
                    "error",
                    &format!("Upload error: {}", e),
                    None,
                ));
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Upload error: {}", e),
                });
            }
        };

        field_name = field.name().unwrap_or("unknown").to_string();
        let _ = event_tx.send(ProgressEvent::new(
            "uploading",
            "Uploading file...",
            Some(10.0),
        ));

        // Process uploaded chunks
        let mut field_bytes = 0;
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    error!("Error getting next chunk: {}", e);
                    let _ = event_tx.send(ProgressEvent::new(
                        "error",
                        &format!("Upload error: {}", e),
                        None,
                    ));
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Upload error: {}", e),
                    });
                }
            };

            field_bytes += data.len() as u64;
            total_bytes += data.len() as u64;

            // Write chunk to file
            if let Err(e) = file.write_all(&data).await {
                error!("Failed to write to file: {}", e);
                let _ = event_tx.send(ProgressEvent::new(
                    "error",
                    &format!("Error writing to file: {}", e),
                    None,
                ));
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Error writing to file: {}", e),
                });
            }

            // Update progress (scaling between 10-50%)
            if field_bytes % (512 * 1024) == 0 {
                // Update every 512KB
                let progress = 10.0 + (field_bytes as f32 / 1024.0 / 1024.0); // Rough estimate
                let scaled_progress = if progress > 50.0 { 50.0 } else { progress };
                let _ = event_tx.send(ProgressEvent::new(
                    "uploading",
                    &format!("Uploaded: {:.1} MB", field_bytes as f32 / 1024.0 / 1024.0),
                    Some(scaled_progress),
                ));
            }
        }
    }

    if total_bytes == 0 {
        error!("Uploaded file is empty");
        let _ = event_tx.send(ProgressEvent::new("error", "Uploaded file is empty", None));
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Uploaded file is empty".to_string(),
        });
    }

    info!(
        "Upload complete: {} bytes in field '{}'",
        total_bytes, field_name
    );
    let _ = event_tx.send(ProgressEvent::new(
        "uploaded",
        &format!(
            "Upload complete: {:.2} MB",
            total_bytes as f32 / 1024.0 / 1024.0
        ),
        Some(50.0),
    ));

    // Verify file exists and has content before extraction
    match fs::metadata(&archive_path).await {
        Ok(metadata) => {
            let size_bytes = metadata.len();
            if size_bytes == 0 {
                error!("Uploaded file is empty (0 bytes)");
                let _ = event_tx.send(ProgressEvent::new("error", "Uploaded file is empty", None));
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Uploaded file is empty".to_string(),
                });
            }
            info!("Uploaded file size: {} bytes", size_bytes);
        }
        Err(e) => {
            error!("Failed to check uploaded file: {}", e);
            let _ = event_tx.send(ProgressEvent::new(
                "error",
                &format!("Failed to verify uploaded file: {}", e),
                None,
            ));
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to verify uploaded file: {}", e),
            });
        }
    }

    // Extract and cleanup
    if let Err(response) =
        extract_and_cleanup(archive_path, target_directory, event_tx.clone()).await
    {
        return response;
    }

    info!("Successfully extracted uploaded content");

    // Complete installation
    complete_installation(
        event_tx,
        app_state,
        "Successfully uploaded and extracted content",
    )
    .await
}
