use anyhow::Result;
use serde::Deserialize;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::fs;

/// Parse the configuration from a given file
pub async fn parse<P: AsRef<Path>>(path: P) -> Result<Config> {
    let raw = fs::read(path).await?;
    Ok(toml::from_slice(&raw)?)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: SocketAddr,
    pub log: String,
    pub repositories: PathBuf,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    Push { branch: String },
    Release,
}

impl Action {
    /// Checks that the branch is allowed on a push
    pub fn matches(&self, branch: Option<&str>) -> bool {
        match self {
            Self::Push { branch: b } => b == branch.unwrap_or_default(),
            Self::Release => true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Mode {
    All,
    Blacklist { repositories: Vec<String> },
    Whitelist { repositories: Vec<String> },
}

impl Mode {
    /// Checks that the repository name is allowed to be deployed
    #[allow(clippy::ptr_arg)]
    pub fn matches(&self, repository: &String) -> bool {
        match self {
            Self::All => true,
            Self::Blacklist { repositories } => !repositories.contains(repository),
            Self::Whitelist { repositories } => repositories.contains(repository),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub action: Action,
    #[serde(flatten)]
    pub mode: Mode,
}

impl Event {
    /// Checks that the repository configuration is allowed
    #[allow(clippy::ptr_arg)]
    pub fn matches(&self, repository: &String, branch: Option<&str>) -> bool {
        self.mode.matches(repository) && self.action.matches(branch)
    }
}
