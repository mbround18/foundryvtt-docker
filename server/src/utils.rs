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
            resolve_foundry_script_path(&APPLICATION_DIR)
        };
    }

    /// Resolves the path to the Foundry VTT main.js script
    /// Tries the old path (resources/app/main.js) first for compatibility with older versions,
    /// then falls back to the new path (main.js) for newer versions
    pub fn resolve_foundry_script_path(app_dir: &str) -> PathBuf {
        let base = PathBuf::from(app_dir);
        
        // Try old path first (older Foundry VTT versions)
        let old_path = base.join("resources").join("app").join("main.js");
        if old_path.exists() {
            debug!("Using old Foundry VTT path: {:?}", old_path);
            return old_path;
        }
        
        // Fall back to new path (newer Foundry VTT versions)
        let new_path = base.join("main.js");
        debug!("Using new Foundry VTT path: {:?}", new_path);
        new_path
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_foundry_script_path_old_structure() {
        // Create a temporary directory structure for old Foundry VTT
        let temp_dir = std::env::temp_dir().join("foundry_test_old");
        let old_path = temp_dir.join("resources").join("app");
        let main_js = old_path.join("main.js");
        
        // Clean up from previous runs
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Create directory structure and file
        fs::create_dir_all(&old_path).unwrap();
        fs::write(&main_js, "// old main.js").unwrap();
        
        // Test resolution
        let resolved = paths::resolve_foundry_script_path(temp_dir.to_str().unwrap());
        assert_eq!(resolved, main_js);
        assert!(resolved.to_str().unwrap().contains("resources/app/main.js"));
        
        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_resolve_foundry_script_path_new_structure() {
        // Create a temporary directory structure for new Foundry VTT (no resources/app)
        let temp_dir = std::env::temp_dir().join("foundry_test_new");
        let main_js = temp_dir.join("main.js");
        
        // Clean up from previous runs
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Create directory and file (without resources/app)
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(&main_js, "// new main.js").unwrap();
        
        // Test resolution
        let resolved = paths::resolve_foundry_script_path(temp_dir.to_str().unwrap());
        assert_eq!(resolved, main_js);
        assert!(!resolved.to_str().unwrap().contains("resources/app"));
        
        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_resolve_foundry_script_path_neither_exists() {
        // Test with a directory where neither file exists
        let temp_dir = std::env::temp_dir().join("foundry_test_none");
        
        // Clean up from previous runs
        let _ = fs::remove_dir_all(&temp_dir);
        
        // Create empty directory
        fs::create_dir_all(&temp_dir).unwrap();
        
        // Should fall back to new path even if it doesn't exist
        let resolved = paths::resolve_foundry_script_path(temp_dir.to_str().unwrap());
        let expected = temp_dir.join("main.js");
        assert_eq!(resolved, expected);
        assert!(!resolved.exists());
        
        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}

