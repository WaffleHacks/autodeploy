use anyhow::Result;
use serde::Deserialize;
use std::{fs, net::IpAddr, path::Path};

/// Parse the configuration from a given file
pub fn parse<P: AsRef<Path>>(path: P) -> Result<Config> {
    let raw = fs::read(path)?;
    Ok(toml::from_slice(&raw)?)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    server: Server,
    events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    address: IpAddr,
    port: u16,
    secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    Push { branch: String },
    Release,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Mode {
    All,
    Blacklist { repositories: Vec<String> },
    Whitelist { repositories: Vec<String> },
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    action: Action,
    #[serde(flatten)]
    mode: Mode,
}
