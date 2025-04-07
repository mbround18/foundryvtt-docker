use crate::config::AppConfig;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::time::{Duration, sleep};
use tracing::{debug, error, info, warn};

pub async fn launch_foundry_process(
    shutdown_rx: Option<oneshot::Receiver<()>>,
    config: &AppConfig,
) {
    // Convert string args to &str for the launch_foundry function
    let args: Vec<&str> = config.foundry_args.iter().map(|s| s.as_str()).collect();

    // Launch Foundry in the same task, passing the shutdown channel
    launch_foundry(&args, &config.foundry_script, shutdown_rx).await;
}

pub async fn launch_foundry(
    args: &[&str],
    script_path: &str,
    shutdown_rx: Option<oneshot::Receiver<()>>,
) {
    let script_path_owned = script_path.to_string();

    // Take ownership of the shutdown_rx outside the loop
    let mut shutdown_rx_option = shutdown_rx;

    loop {
        // Wait until the script file is present
        if !Path::new(&script_path_owned).exists() {
            warn!("⚠️ Script not found at {}, waiting...", script_path_owned);
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        info!("🚀 Launching FoundryVTT with script: {}", script_path_owned);
        debug!(
            "Launch command: npx --yes node {} with args: {:?}",
            script_path_owned, args
        );

        let mut cmd = Command::new("npx");
        cmd.arg("--yes")
            .arg("node")
            .arg(&script_path_owned)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        debug!("Full command: {:?}", cmd);

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                error!("❌ Failed to spawn FoundryVTT: {}", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("FoundryVTT process started");

        // Handle shutdown signal if provided
        if let Some(shutdown_rx) = shutdown_rx_option.take() {
            let child_id = child.id();
            tokio::select! {
                exit_status = child.wait() => {
                    match exit_status {
                        Ok(exit) => {
                            warn!("⚠️ FoundryVTT exited with: {}", exit);
                        }
                        Err(e) => {
                            error!("❌ Failed to wait for FoundryVTT: {}", e);
                        }
                    }
                },
                _ = shutdown_rx => {
                    info!("Received shutdown signal, terminating FoundryVTT process");
                    if let Some(pid) = child_id {
                        info!("Sending SIGTERM to FoundryVTT process (PID: {})", pid);
                        if let Err(e) = child.kill().await {
                            error!("Failed to kill FoundryVTT process: {}", e);
                        }
                    }
                    // Wait for child process to exit after kill signal
                    if let Err(e) = child.wait().await {
                        error!("Error waiting for FoundryVTT to exit: {}", e);
                    }
                    info!("FoundryVTT process terminated");
                    return; // Exit the function, don't restart
                }
            }
        } else {
            // Without shutdown channel, just wait for the process
            match child.wait().await {
                Ok(exit) => {
                    warn!("⚠️ FoundryVTT exited with: {}", exit);
                }
                Err(e) => {
                    error!("❌ Failed to wait for FoundryVTT: {}", e);
                }
            }
        }

        // Retry after 5 seconds if the script or process exits (only if we didn't get a shutdown signal)
        sleep(Duration::from_secs(5)).await;
    }
}
