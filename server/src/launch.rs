// launch.rs
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use std::process::Stdio;
use std::path::Path;

pub async fn launch_foundry(args: &[&str], script_path: &str) {
    let script_path_owned = script_path.to_string();
    loop {
        // Wait until the script file is present
        if !Path::new(&script_path_owned).exists() {
            tracing::warn!("⚠️ Script not found at {}, waiting...", script_path_owned);
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        tracing::info!("🚀 Launching FoundryVTT with script: {}", script_path_owned);
        let status = Command::new("npx")
            .arg("--yes")
            .arg("node")
            .arg(&script_path_owned)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn FoundryVTT")
            .wait()
            .await;

        match status {
            Ok(exit) => {
                tracing::warn!("⚠️ FoundryVTT exited with: {}", exit);
            }
            Err(e) => {
                tracing::error!("❌ Failed to wait for FoundryVTT: {}", e);
            }
        }

        // Retry after 5 seconds if the script or process exits
        sleep(Duration::from_secs(5)).await;
    }
}
