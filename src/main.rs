use anyhow::{Context, Result};
use std::env;
use tokio::fs;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

mod config;
mod github;
mod http;
mod repo;

#[tokio::main]
async fn main() -> Result<()> {
    // Determine what config file to load
    let file = env::var("AUTODEPLOY_CONFIG").unwrap_or_else(|_| "./config.toml".to_string());

    // Get the configuration
    let configuration = config::parse(file)
        .await
        .context("Failed to load configuration")?;
    let address = configuration.server.address;
    let log_filter = env::var("RUST_LOG").unwrap_or_else(|_| configuration.server.log.clone());

    // Ensure the directory for the repositories exists
    if !configuration.server.repositories.exists() {
        fs::create_dir_all(&configuration.server.repositories)
            .await
            .context("Failed to create repository directory")?;
    }

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // Setup the routes and launch the server
    let routes = http::routes(configuration)
        .recover(http::recover)
        .with(warp::trace::request());
    warp::serve(routes).run(address).await;

    Ok(())
}
