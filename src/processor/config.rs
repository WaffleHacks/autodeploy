use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    deploy: Vec<Action>,
}

impl Config {
    /// Parse the configuration from the repository
    pub async fn parse(path: &Path) -> Result<Vec<Action>> {
        // Read and parse the config
        let content = fs::read(path).await.context("failed to find config")?;
        let config: Config = toml::from_slice(&content).context("invalid config format")?;

        // Only get the deployment actions
        Ok(config.deploy)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    Command { command: String, args: Vec<String> },
    Copy { src: PathBuf, dest: PathBuf },
}
