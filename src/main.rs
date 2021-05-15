use anyhow::{Context, Result};
use std::env;
use tokio::fs;
use tracing::Span;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{
    trace::{trace, Info, Trace},
    Filter,
};

mod config;
mod github;
mod http;
mod processor;
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

    // Create the processing runner
    let sender = processor::create(configuration.server.workers);

    // Setup the routes and launch the server
    let routes = http::routes(configuration, sender)
        .recover(http::recover)
        .with(trace_request());
    warp::serve(routes).run(address).await;

    Ok(())
}

/// Wrap the request with some information allowing it
/// to be traced through the logs. Built off of the
/// `warp::trace::request` implementation
fn trace_request() -> Trace<impl Fn(Info) -> Span + Clone> {
    use tracing::field::{display, Empty};

    trace(|info: Info| {
        let span = tracing::info_span!(
            "request",
            remote.addr = Empty,
            method = %info.method(),
            path = %info.path(),
            version = ?info.version(),
            referrer = Empty,
            id = %uuid::Uuid::new_v4(),
        );

        // Record optional fields
        if let Some(remote_addr) = info.remote_addr() {
            span.record("remote.addr", &display(remote_addr));
        }
        if let Some(referrer) = info.referer() {
            span.record("referrer", &display(referrer));
        }

        tracing::debug!(parent: &span, "received request");

        span
    })
}
