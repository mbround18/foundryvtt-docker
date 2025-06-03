use crate::utils::paths;
use std::env;

pub struct AppConfig {
    pub static_files_dir: String,
    pub server_port: u16,
    pub server_host: String,
    pub target_dir: String,
    pub foundry_args: Vec<String>,
    pub foundry_script: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let static_files_dir =
            env::var("STATIC_FILES_DIR").unwrap_or_else(|_| "static".to_string());

        let server_port = env::var("SERVER_PORT")
            .or_else(|_| env::var("APPLICATION_PORT"))
            .unwrap_or_else(|_| "4444".to_string())
            .parse::<u16>()
            .unwrap_or(4444);

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let target_dir = get_target_directory();

        let foundry_host =
            env::var("APPLICATION_HOST").unwrap_or("foundry.vtt".to_string());

        let foundry_args = vec![
            format!("--dataPath={}", *paths::DATA_DIR),
            format!("--port={}", server_port),
            format!("--hostname={}", foundry_host),
            "--noupnp".to_string(),
            "--proxySSL".to_string(),
        ];

        let foundry_script = paths::FOUNDRY_SCRIPT_PATH.to_string_lossy().to_string();

        Self {
            static_files_dir,
            server_port,
            server_host,
            target_dir,
            foundry_args,
            foundry_script,
        }
    }
}

pub(crate) fn get_target_directory() -> String {
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
