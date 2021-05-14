use super::{
    errors::{BodyParsingError, SignatureError},
    SharedConfig,
};
use crate::github::{Github, ReleaseAction};
use bytes::Bytes;
use ring::hmac;
use tracing::info;
use warp::{http::StatusCode, reject, Rejection, Reply};

/// Ensure that the signature from github is valid
fn validate_signature(
    raw_body: &[u8],
    raw_signature: String,
    secret: &[u8],
) -> Result<(), Rejection> {
    // Remove the sha256 prefix from the hash
    let signature_hex = raw_signature
        .strip_prefix("sha256=")
        .ok_or(reject::custom(SignatureError))?;
    let signature = hex::decode(signature_hex).map_err(|_| reject::custom(SignatureError))?;

    // Check if the signature is valid
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    Ok(hmac::verify(&key, raw_body.as_ref(), &signature)
        .map_err(|_| reject::custom(SignatureError))?)
}

/// Handle receiving webhooks from GitHub
pub async fn hook(
    raw_body: Bytes,
    raw_signature: String,
    config: SharedConfig,
) -> Result<impl Reply, Rejection> {
    // Ensure the signature is valid
    validate_signature(&raw_body, raw_signature, config.server.secret.as_bytes())?;

    // Attempt to parse the body
    let body: Github =
        serde_json::from_slice(&raw_body).map_err(|_| reject::custom(BodyParsingError))?;
    info!("got new {} hook", body.name());

    // Operate based on the body type
    match body {
        Github::Ping { zen, hook_id } => info!("received ping from hook {}: {}", hook_id, zen),
        Github::Push {
            reference,
            repository,
        } => {}
        Github::Release {
            action,
            repository,
            release,
        } => {}
    }

    Ok(StatusCode::NO_CONTENT)
}
