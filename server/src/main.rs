#[macro_use]
extern crate rocket;

use reqwest::Client;
use rocket::fs::{ FileServer};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::copy;
use std::path::Path;
use tokio::task;
use zip::read::ZipArchive;

#[derive(Deserialize)]
struct UrlPayload {
    url: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SuccessResponse {
    message: String,
}

fn get_target_directory() -> String {
    env::var("APPLICATION_DIR").unwrap_or_else(|_| {
        let mut dir = env::current_dir().expect("Failed to get current directory");
        dir.push("tmp");
        dir.to_str().unwrap().to_string()
    })
}

#[post("/download", format = "json", data = "<url_payload>")]
async fn download(url_payload: Json<UrlPayload>) -> Json<SuccessResponse> {
    let url = url_payload.url.clone();

    // Perform the download in an asynchronous context
    let response_bytes = Client::new()
        .get(&url)
        .send()
        .await
        .expect("Failed to send request")
        .bytes()
        .await
        .expect("Failed to get response bytes");

    // Determine the target directory
    let target_directory = get_target_directory();

    // Create the target directory if it doesn't exist
    if !Path::new(&target_directory).exists() {
        fs::create_dir_all(&target_directory).expect("Failed to create target directory");
    }

    // Perform the file operations and extraction in a blocking task
    task::spawn_blocking(move || {
        let archive_path = format!("{}/archive.zip", target_directory);
        let mut dest = File::create(&archive_path).expect("Failed to create file");
        copy(&mut response_bytes.as_ref(), &mut dest).expect("Failed to copy content");

        // Unzip the file
        let file = File::open(&archive_path).expect("Failed to open file");
        let mut archive = ZipArchive::new(file).expect("Failed to read zip archive");
        archive
            .extract(&target_directory)
            .expect("Failed to extract zip archive");

        // Exit the process
        std::process::exit(0);
    })
    .await
    .expect("The blocking task panicked");

    Json(SuccessResponse {
        message: format!("Downloaded and extracted the content from: {}", url),
    })
}

#[launch]
fn rocket() -> _ {
    let static_files_dir = env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "static".to_string());
    println!("Serving static files from: {}", static_files_dir);
    println!("Downloading files to: {}", get_target_directory());
    rocket::build()
        .mount("/", FileServer::from(static_files_dir))
        .mount("/", routes![download])
}