use anyhow::{Context, Result};
use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};
use tracing::debug;

pub mod paths {
    use super::*;

    lazy_static! {
        /// Base application directory where the Foundry VTT application is installed
        pub static ref APPLICATION_DIR: String = env::var("APPLICATION_DIR")
            .unwrap_or_else(|_| "/foundryvtt".to_string());

        /// Data directory for user data
        pub static ref DATA_DIR: String = env::var("DATA_DIR")
            .unwrap_or_else(|_| "/foundrydata".to_string());

        /// Path to the main Foundry script
        pub static ref FOUNDRY_SCRIPT_PATH: PathBuf = {
            let mut path = PathBuf::from(&*APPLICATION_DIR);
            path.push("main.js");
            path
        };
    }
}

/// Run a system command and return its output
pub fn run_command(command: &str, args: &[&str]) -> Result<String> {
    debug!("Running command: {} {:?}", command, args);

    let output: Output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute command: {} {:?}", command, args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!(
            "Command failed with status code {:?}: {}",
            output.status.code(),
            stderr.trim()
        );
    }

    // Return stdout as string
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
