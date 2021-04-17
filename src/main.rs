use anyhow::{Context, Result};
use std::env;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

mod config;
mod github;
mod http;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the configuration
    let configuration = config::parse("./config.toml").context("Failed to load configuration")?;
    let address = configuration.server.address.clone();
    let log_filter = env::var("RUST_LOG").unwrap_or(configuration.server.log.clone());

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
