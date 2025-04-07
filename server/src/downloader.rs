use reqwest::Client;
use tokio::fs;
use tracing::{debug, error, info};

pub struct DownloadService;

impl DownloadService {
    /// Download file from `url` and write it to `save_path`, streaming the response.
    pub async fn download_file_from_url(
        url: &str,
        save_path: &str,
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

        // Log content length if available
        if let Some(content_length) = resp.content_length() {
            debug!("Content length: {} bytes", content_length);
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
            // Log progress every 5MB
            if downloaded % (5 * 1024 * 1024) < chunk.len() as u64 {
                debug!("Downloaded: {} MB", downloaded / (1024 * 1024));
            }
        }

        info!("Download completed successfully: {} bytes", downloaded);
        Ok(())
    }
}
