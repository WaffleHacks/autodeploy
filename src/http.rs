use serde::Serialize;
use std::convert::Infallible;
use warp::{
    filters::body::BodyDeserializeError, http::StatusCode, reject::MethodNotAllowed, reply, Filter,
    Rejection, Reply,
};

/// An API error serializable to JSON
#[derive(Serialize)]
struct Error {
    code: u16,
    message: String,
}

/// Build the routes for the API
pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Health check route
    let health = warp::path("health")
        .and(warp::get())
        .map(|| StatusCode::NO_CONTENT);

    // Main hook route
    let hook = warp::path::end()
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(handle);

    health.or(hook)
}

/// Handle receiving webhooks from GitHub
// TODO: add body parsing
async fn handle(body: String) -> Result<impl Reply, Rejection> {
    Ok(reply::json(&"hello".to_string()))
}

/// Convert a `Rejection` to an API error, otherwise simply passes
/// the rejection along.
pub async fn recover(error: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if error.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "not found";
    } else if let Some(_) = error.find::<BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "bad request";
    } else if let Some(_) = error.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "method not allowed";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "unhandled rejection";
    }

    // Build the response
    let json = reply::json(&Error {
        code: code.as_u16(),
        message: message.into(),
    });
    Ok(reply::with_status(json, code))
}
