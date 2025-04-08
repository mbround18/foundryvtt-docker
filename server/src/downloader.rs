use crate::events::ProgressEvent;
use reqwest::Client;
use tokio::fs;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

pub struct DownloadService;

impl DownloadService {
    /// Download file from `url` and write it to `save_path`, streaming the response.
    pub async fn download_file_from_url(
        url: &str,
        save_path: &str,
        event_tx: broadcast::Sender<ProgressEvent>,
    ) -> Result<(), actix_web::Error> {
        info!("Starting download from URL: {}", url);

        let client = Client::new();
        let mut resp = client.get(url).send().await.map_err(|e| {
            error!("Request error: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to send request: {}", e))
        })?;

        // Check the response status
        if !resp.status().is_success() {
            let status = resp.status();
            error!("Download request failed with status: {}", status);
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Download failed with status: {}",
                status
            )));
        }

        // Get content length if available for progress calculation
        let content_length = resp.content_length().unwrap_or(0);
        if content_length > 0 {
            debug!("Content length: {} bytes", content_length);
            let _ = event_tx.send(ProgressEvent::new(
                "downloading",
                &format!("Download size: {} MB", content_length / (1024 * 1024)),
                Some(15.0),
            ));
        } else {
            let _ = event_tx.send(ProgressEvent::new(
                "downloading",
                "Download size unknown",
                Some(15.0),
            ));
        }

        let mut out = fs::File::create(save_path).await.map_err(|e| {
            error!("Failed to create file: {}", e);
            actix_web::error::ErrorInternalServerError(format!("Failed to create file: {}", e))
        })?;
        info!("Saving downloaded file to: {}", save_path);

        // Use a buffer to track download progress
        let mut downloaded: u64 = 0;
        while let Some(chunk) = resp.chunk().await.map_err(|e| {
            error!("Failed reading download stream: {}", e);
            actix_web::error::ErrorInternalServerError(format!(
                "Failed reading download stream: {}",
                e
            ))
        })? {
            use tokio::io::AsyncWriteExt;
            out.write_all(&chunk).await.map_err(|e| {
                error!("Failed to write file: {}", e);
                actix_web::error::ErrorInternalServerError(format!("Failed to write file: {}", e))
            })?;

            downloaded += chunk.len() as u64;

            // Calculate progress between 15-50% for download phase
            if content_length > 0 {
                let progress_percent = (downloaded as f64 / content_length as f64) * 100.0;
                let normalized_progress = 15.0 + (progress_percent * 0.35);

                // Log progress every 5MB or 10% progress
                if downloaded % (5 * 1024 * 1024) < chunk.len() as u64
                    || progress_percent % 10.0
                        < (chunk.len() as f64 / content_length as f64 * 100.0)
                {
                    debug!(
                        "Downloaded: {} MB ({:.1}%)",
                        downloaded / (1024 * 1024),
                        progress_percent
                    );
                    let _ = event_tx.send(ProgressEvent::new(
                        "downloading",
                        &format!(
                            "Downloaded: {:.1} MB ({:.0}%)",
                            downloaded as f32 / (1024.0 * 1024.0),
                            progress_percent
                        ),
                        Some(normalized_progress as f32),
                    ));
                }
            } else if downloaded % (5 * 1024 * 1024) < chunk.len() as u64 {
                // If content length is unknown, just show downloaded amount
                debug!("Downloaded: {} MB", downloaded / (1024 * 1024));
                let _ = event_tx.send(ProgressEvent::new(
                    "downloading",
                    &format!(
                        "Downloaded: {:.1} MB",
                        downloaded as f32 / (1024.0 * 1024.0)
                    ),
                    Some(30.0),
                ));
            }
        }

        info!("Download completed successfully: {} bytes", downloaded);
        Ok(())
    }
}
