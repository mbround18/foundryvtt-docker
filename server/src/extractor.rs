use std::fs::File;
use std::path::Path;
use tokio::fs;
use tokio::task;
use tracing::{debug, error, info, warn};
use zip::read::ZipArchive;

pub struct ExtractorService;

impl ExtractorService {
    /// Extract ZIP at `archive_path` into `target_directory` using a blocking task.
    pub async fn extract_zip(
        archive_path: String,
        target_directory: String,
    ) -> Result<(), std::io::Error> {
        info!("Starting extraction of archive: {}", archive_path);

        // Verify file exists before attempting extraction
        if !Path::new(&archive_path).exists() {
            let err_msg = format!("Archive file not found at path: {}", archive_path);
            error!("{}", err_msg);
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, err_msg));
        }

        // Ensure target directory exists
        if !Path::new(&target_directory).exists() {
            info!("Creating target directory: {}", &target_directory);
            fs::create_dir_all(&target_directory).await?;
        }

        // Get file size for logging
        match fs::metadata(&archive_path).await {
            Ok(metadata) => {
                let size_mb = metadata.len() as f64 / 1_048_576.0;
                debug!("Archive file size: {:.2} MB", size_mb);
            }
            Err(e) => warn!("Could not get archive file size: {}", e),
        }

        // Clone the target_directory for use after the spawn_blocking closure
        let target_directory_clone = target_directory.clone();

        task::spawn_blocking(move || {
            // Print normalized paths for debugging
            let archive_path_obj = Path::new(&archive_path);
            let target_dir_obj = Path::new(&target_directory);

            debug!(
                "Extracting from canonical path: {}",
                archive_path_obj.display()
            );
            debug!("Extracting to canonical path: {}", target_dir_obj.display());

            // Open and extract the file
            let file = match File::open(&archive_path) {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to open archive file: {}", e);
                    return Err(e);
                }
            };

            let mut archive = match ZipArchive::new(file) {
                Ok(a) => {
                    debug!("Successfully opened ZIP archive with {} files", a.len());
                    a
                }
                Err(e) => {
                    error!("Failed to read ZIP archive: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid ZIP file: {}", e),
                    ));
                }
            };

            // Extract with detailed error information
            match archive.extract(&target_directory) {
                Ok(()) => {
                    debug!("ZIP extraction completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to extract archive: {}", e);
                    // Convert ZipError to std::io::Error
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Extraction failed: {}", e),
                    ))
                }
            }
        })
        .await
        .unwrap_or_else(|e| {
            error!("Blocking task panicked during extraction: {}", e);
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Extraction thread panicked: {}", e),
            ))
        })?;

        info!(
            "Extraction completed successfully to {}",
            target_directory_clone
        );
        Ok(())
    }
}
