use git2::Error as Git2Error;
use serde::Serialize;
use std::convert::Infallible;
use tracing::error;
use warp::{
    http::StatusCode,
    reject::{MethodNotAllowed, MissingHeader, Reject},
    reply, Rejection, Reply,
};

/// An API error serializable to JSON
#[derive(Serialize)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

/// Raised when the signature is invalid or cannot be processed
#[derive(Debug)]
pub struct SignatureError;
impl Reject for SignatureError {}

/// Raised when the body cannot be parsed
#[derive(Debug)]
pub struct BodyParsingError;
impl Reject for BodyParsingError {}

/// Raised when the repo is not allowed to be deployed
#[derive(Debug)]
pub struct UndeployableError;
impl Reject for UndeployableError {}

/// Raised when there is an error interacting with the git repository
#[derive(Debug)]
pub struct GitError(pub Git2Error);
impl Reject for GitError {}

/// Convert a `Rejection` to an API error, otherwise simply passes
/// the rejection along.
pub async fn recover(error: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if error.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "not found";
    } else if error.find::<MissingHeader>().is_some() || error.find::<BodyParsingError>().is_some()
    {
        code = StatusCode::BAD_REQUEST;
        message = "bad request";
    } else if error.find::<MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "method not allowed";
    } else if error.find::<SignatureError>().is_some() {
        code = StatusCode::UNAUTHORIZED;
        message = "unauthorized";
    } else if error.find::<UndeployableError>().is_some() {
        code = StatusCode::FORBIDDEN;
        message = "forbidden";
    } else if let Some(e) = error.find::<GitError>() {
        error!(
            "error while interacting with local repo: ({:?}, {:?}) {}",
            e.0.class(),
            e.0.code(),
            e.0.message()
        );
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "unhandled rejection"
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
