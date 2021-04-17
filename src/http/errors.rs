use serde::Serialize;
use std::convert::Infallible;
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

/// Convert a `Rejection` to an API error, otherwise simply passes
/// the rejection along.
pub async fn recover(error: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if error.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "not found";
    } else if let Some(_) = error.find::<MissingHeader>() {
        code = StatusCode::BAD_REQUEST;
        message = "bad request";
    } else if let Some(_) = error.find::<BodyParsingError>() {
        code = StatusCode::BAD_REQUEST;
        message = "bad request";
    } else if let Some(_) = error.find::<MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "method not allowed";
    } else if let Some(_) = error.find::<SignatureError>() {
        code = StatusCode::FORBIDDEN;
        message = "forbidden";
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
