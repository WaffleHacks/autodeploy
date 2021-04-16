use anyhow::{Context, Result};
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the configuration
    let configuration = config::parse("./config.toml").context("Failed to load configuration")?;

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // TODO: remove when routes are implemented
    let hello = warp::path("hello")
        .and(warp::get())
        .map(|| {
            info!("Hello world");
            "Hello world"
        })
        .with(warp::trace::named("hello"));

    // Setup the routes and launch the server
    let routes = hello.with(warp::trace::request());
    warp::serve(routes).run(configuration.server.address).await;

    Ok(())
}
