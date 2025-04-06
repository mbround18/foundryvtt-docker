// main.rs
mod launch;

use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::Path;
use tokio::fs;
use tokio::task;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_subscriber;
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
    env::var("APPLICATION_DIR").unwrap_or_else(|_| {
        let mut dir = env::current_dir().expect("Failed to get current directory");
        dir.push("tmp");
        dir.to_str().unwrap().to_string()
    })
}

/// Download file from `url` and write it to `save_path`, streaming the response.
async fn download_file_from_url(url: &str, save_path: &str) -> Result<(), actix_web::Error> {
    let mut resp = Client::new().get(url).send().await.map_err(|e| {
        error!("Request error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to send request")
    })?;

    let mut out = fs::File::create(save_path)
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create file"))?;

    while let Some(chunk) = resp.chunk().await.map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed reading download stream")
    })? {
        use tokio::io::AsyncWriteExt;
        out.write_all(&chunk)
            .await
            .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to write file"))?;
    }

    Ok(())
}

/// Extract ZIP at `archive_path` into `target_directory` using a blocking task.
async fn extract_zip(archive_path: String, target_directory: String) -> Result<(), std::io::Error> {
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
        })
}

// ---------- Handlers ----------

async fn download_and_extract(url_payload: web::Json<UrlPayload>) -> impl Responder {
    let url = url_payload.url.clone();
    let target_directory = get_target_directory();

    // Ensure target directory exists
    if !Path::new(&target_directory).exists() {
        if let Err(e) = fs::create_dir_all(&target_directory).await {
            error!("Failed to create target directory: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
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

    // Cleanup
    if let Err(e) = fs::remove_file(&archive_path).await {
        error!("Failed to delete temporary zip file: {}", e);
    }

    HttpResponse::Ok().json(SuccessResponse {
        message: format!("Downloaded and extracted content from: {}", url),
    })
}

/// Proxy requests to `localhost:4444`.
async fn proxy(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let forward_url = format!("http://localhost:4444{}", req.uri());
    let client = awc::Client::default();

    match client.request_from(forward_url, req.head()).send_body(body).await {
        Ok(mut res) => {
            let mut client_resp = HttpResponse::build(res.status());
            for (key, value) in res.headers() {
                client_resp.insert_header((key.clone(), value.clone()));
            }
            client_resp.body(res.body().await.unwrap_or_default())
        }
        Err(e) => {
            error!("Proxy error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let foundry_args = vec![
        "--dataPath=/foundrydata",
        "--port=4444",
        "--hostname=foundry.vtt",
        "--noupnp",
        "--proxySSL",
    ];

    let foundry_script = "/foundryvtt/resources/app/main.js".to_string();

    // Move into the spawn block
    tokio::spawn(async move {
        launch::launch_foundry(&foundry_args, &foundry_script).await;
    });


    let static_files_dir = env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "static".to_string());
    info!("Serving static files from: {}", static_files_dir);
    info!("Downloading files to: {}", get_target_directory());

    HttpServer::new(move || {
        App::new()
            // Logging for Actix
            .wrap(TracingLogger::default())
            // Download route
            .route("/download", web::post().to(download_and_extract))
            // Serve static files
            .service(Files::new("/", &static_files_dir).index_file("index.html"))
            // Catch-all proxy to Foundry
            .route("/{tail:.*}", web::to(proxy))
    })
        .bind(("0.0.0.0", 4444))?
        .run()
        .await
}
