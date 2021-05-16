use super::{
    config::{Action, Config},
    Message,
};
use anyhow::Result;
use async_channel::Receiver;
use std::path::Path;
use tokio::{fs, process::Command};
use tracing::{error, info, instrument};

/// Process incoming deployment workloads
#[instrument(skip(receiver))]
pub async fn worker(id: u32, receiver: Receiver<Message>) {
    info!("started worker {}", id);

    while let Ok(message) = receiver.recv().await {
        info!("beginning deploy");

        // Run the deployment
        let result = deploy(&message.path, &message.repository).await;
        match result {
            Ok(true) => info!("deploy successful"),
            Ok(false) => error!("deploy failed"),
            Err(e) => error!(error = %e, "deploy failed"),
        }
    }
}

/// Run the deployment process
#[instrument(skip(path))]
async fn deploy(path: &Path, repository: &str) -> Result<bool> {
    // Get the deployment configuration
    let actions = Config::parse(&path.join("autodeploy.toml")).await?;
    info!("successfully parsed configuration");

    // Run the actions
    let mut successful = 0;
    for action in &actions {
        match action {
            Action::Command { command, args } => {
                info!(command = %&command, "running command");

                // Build the command
                let mut cmd = Command::new(&command);
                for arg in args {
                    cmd.arg(arg);
                }

                // Get the output of the command
                let status = cmd.status().await?;
                if status.success() {
                    info!(command = %&command, "command succeeded");
                } else {
                    error!(command = %&command, code = %status, "command failed");
                    break;
                }

                successful += 1;
            }
            Action::Copy { src, dest } => {
                info!(src = ?&src, dest = ?&dest, "copying file");

                // Copy the file
                let result = fs::copy(path.join(&src), &dest).await;

                // Check for errors
                if let Err(e) = result {
                    error!(src = ?&src, dest = ?&dest, error = %e, "failed to copy file");
                    break;
                }

                successful += 1;
            }
        }
    }

    Ok(successful == actions.len())
}
