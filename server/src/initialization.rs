use anyhow::{Context, Result, anyhow};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::config::AppConfig;
use crate::utils::{paths, run_command};

pub fn initialize(app_config: &AppConfig) -> Result<()> {
    print_banner()?;
    print_system_info()?;
    check_required_env()?;
    validate_env()?;
    ensure_directories()?;

    info!("Configuration Summary:");
    info!("  - Application directory: {}", app_config.target_dir);
    info!("  - Data directory: {}", *paths::DATA_DIR);
    info!(
        "  - Host: {}",
        env::var("APPLICATION_HOST").unwrap_or_else(|_| "foundry.vtt".to_string())
    );
    info!(
        "  - SSL Proxy: {}",
        env::var("SSL_PROXY").unwrap_or_else(|_| "false".to_string())
    );
    info!(
        "  - Port: {}",
        env::var("APPLICATION_PORT").unwrap_or_else(|_| "4444".to_string())
    );

    info!("──────────────────────────────────────────────────────────");
    Ok(())
}

fn print_banner() -> Result<()> {
    info!("──────────────────────────────────────────────────────────");
    info!(
        "🎲 FoundryVTT - {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    info!("──────────────────────────────────────────────────────────");
    Ok(())
}

fn print_system_info() -> Result<()> {
    // Collect system information in a more compact format
    let hostname = run_command("hostname", &[])?.trim().to_string();
    let kernel = run_command("uname", &["-r"])?.trim().to_string();
    let os = run_command(
        "sh",
        &[
            "-c",
            "grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '\"'",
        ],
    )?
    .trim()
    .to_string();
    let cpu = run_command(
        "sh",
        &[
            "-c",
            "lscpu | grep 'Model name' | cut -d: -f2 | sed 's/^ *//'",
        ],
    )?
    .trim()
    .to_string();
    let memory = run_command("sh", &["-c", "free -h | awk '/^Mem:/ {print $2}'"])?
        .trim()
        .to_string();
    let disk = run_command("sh", &["-c", "df -h / | awk 'NR==2 {print $4}'"])?
        .trim()
        .to_string();
    let node_version = run_command("node", &["--version"])?.trim().to_string();
    let npm_version = run_command("npm", &["--version"])?.trim().to_string();

    info!("System Information:");
    info!("  - Hostname: {}", hostname);
    info!("  - OS: {} (Kernel: {})", os, kernel);
    info!(
        "  - Resources: CPU: {}, Memory: {}, Free Disk: {}",
        if cpu.is_empty() { "Unknown" } else { &cpu },
        memory,
        disk
    );
    info!("  - Node: v{}, NPM: v{}", node_version, npm_version);

    debug!("Detailed System Information:");
    debug!("  - Hostname: {}", hostname);
    debug!("  - Kernel: {}", kernel);
    debug!("  - OS: {}", os);
    debug!("  - CPU: {}", if cpu.is_empty() { "Unknown" } else { &cpu });
    debug!("  - Memory: {}", memory);
    debug!("  - Disk Space: {}", disk);
    debug!("  - Node Version: {}", node_version);
    debug!("  - NPM Version: {}", npm_version);

    Ok(())
}

fn check_required_env() -> Result<()> {
    let required_vars = ["APPLICATION_DIR", "DATA_DIR", "APPLICATION_HOST"];
    let mut missing = false;

    info!("Checking environment variables");
    for var in required_vars {
        match env::var(var) {
            Ok(value) => debug!("{} = {}", var, value),
            Err(_) => {
                error!("{} is required but not set!", var);
                missing = true;
            }
        }
    }

    if missing {
        return Err(anyhow!("Missing required environment variables"));
    }

    Ok(())
}

fn validate_env() -> Result<()> {
    let app_dir = &*paths::APPLICATION_DIR;
    let data_dir = &*paths::DATA_DIR;

    if app_dir == data_dir {
        error!("APPLICATION_DIR and DATA_DIR cannot be the same!");
        error!("   Application: {}", app_dir);
        error!("   Data: {}", data_dir);
        return Err(anyhow!("APPLICATION_DIR and DATA_DIR cannot be the same"));
    }

    let app_port = env::var("APPLICATION_PORT").unwrap_or_else(|_| "4444".to_string());
    if app_port.parse::<u32>().is_err() {
        error!("APPLICATION_PORT must be a number: {}", app_port);
        return Err(anyhow!("Invalid APPLICATION_PORT"));
    }

    Ok(())
}

fn ensure_directories() -> Result<()> {
    info!("Validating directories");

    let app_dir = &*paths::APPLICATION_DIR;
    let data_dir = &*paths::DATA_DIR;

    for dir in &[app_dir, data_dir] {
        let path = Path::new(dir);

        if !path.exists() {
            info!("Creating directory: {} (missing)", dir);
            fs::create_dir_all(path).with_context(|| format!("Failed to create {}", dir))?;
        }

        // Check if directory is writable
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();
        let is_writable = permissions.mode() & 0o200 != 0;

        if !is_writable {
            warn!("Directory not writable: {}. This might cause issues.", dir);
        }

        // Print detailed directory info only at debug level
        debug!("Directory details for {}:", dir);
        debug!(
            "  - Status: {}",
            if path.exists() {
                "Exists"
            } else {
                "Does not exist"
            }
        );
        debug!("  - Writable: {}", if is_writable { "Yes" } else { "No" });

        if path.is_dir() {
            let file_count = fs::read_dir(path)?
                .filter(|entry| entry.as_ref().map(|e| e.path().is_file()).unwrap_or(false))
                .count();

            let dir_count = fs::read_dir(path)?
                .filter(|entry| entry.as_ref().map(|e| e.path().is_dir()).unwrap_or(false))
                .count();

            debug!(
                "  - Contents: {} files, {} directories",
                file_count, dir_count
            );
        }
    }

    // Network configuration - simplified for INFO level
    let user_info = run_command("id", &["-u"])?.trim().to_string();
    info!("Running as UID: {}", user_info);

    // Network configuration at debug level
    debug!("Network configuration:");
    if Command::new("ip").arg("addr").arg("show").status().is_ok() {
        let network_info = run_command("ip", &["addr", "show"])?;
        // Just log that we have this info, full details in debug
        debug!("{}", network_info);
    }

    if Command::new("netstat").arg("-tulpn").status().is_ok() {
        let ports_info = run_command("netstat", &["-tulpn"])?;
        debug!("{}", ports_info);
    } else if Command::new("ss").arg("-tulpn").status().is_ok() {
        let ports_info = run_command("ss", &["-tulpn"])?;
        debug!("{}", ports_info);
    }

    Ok(())
}
