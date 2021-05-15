use crate::{config::Config, processor::Message};
use async_channel::Sender;
use std::{convert::Infallible, sync::Arc};
use tracing::info;
use warp::{http::StatusCode, Filter, Rejection, Reply};

mod access;
mod errors;
mod handlers;

pub use errors::recover;

type SharedConfig = Arc<Config>;

fn with_config(
    config: Config,
) -> impl Filter<Extract = (SharedConfig,), Error = Infallible> + Clone {
    let config = Arc::new(config);
    warp::any().map(move || config.clone())
}

fn with_sender(
    sender: Sender<Message>,
) -> impl Filter<Extract = (Sender<Message>,), Error = Infallible> + Clone {
    warp::any().map(move || sender.clone())
}

/// Build the routes for the API
pub fn routes(
    config: Config,
    sender: Sender<Message>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Health check route
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            info!("alive and healthy!");
            StatusCode::NO_CONTENT
        })
        .with(warp::trace::named("health"));

    // Main hook route
    let hook = warp::path::end()
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::bytes())
        .and(warp::header::<String>("X-Hub-Signature-256"))
        .and(with_config(config))
        .and(with_sender(sender))
        .and_then(handlers::hook)
        .with(warp::trace::named("hook"));

    health.or(hook)
}
