use super::{
    errors::{BodyParsingError, SignatureError},
    SharedConfig,
};
use crate::github::Github;
use bytes::Bytes;
use ring::hmac;
use tracing::info;
use warp::{http::StatusCode, reject, Rejection, Reply};

/// Handle receiving webhooks from GitHub
pub async fn hook(
    raw_body: Bytes,
    raw_signature: String,
    config: SharedConfig,
) -> Result<impl Reply, Rejection> {
    // Remove the sha256 prefix from the hash
    let signature_hex = raw_signature
        .strip_prefix("sha256=")
        .ok_or(reject::custom(SignatureError))?;
    let signature = hex::decode(signature_hex).map_err(|_| reject::custom(SignatureError))?;

    // Check if the signature is valid
    let key = hmac::Key::new(hmac::HMAC_SHA256, config.server.secret.as_bytes());
    hmac::verify(&key, raw_body.as_ref(), &signature)
        .map_err(|_| reject::custom(SignatureError))?;

    // Attempt to parse the body
    let body: Github =
        serde_json::from_slice(&raw_body).map_err(|_| reject::custom(BodyParsingError))?;
    info!("got new {} hook", body.name());

    // Operate based on the body type
    match body {
        Github::Ping { zen, hook_id } => info!("received ping from hook {}: {}", hook_id, zen),
        Github::Push { .. } => {}
        Github::Release { .. } => {}
    }

    Ok(StatusCode::NO_CONTENT)
}
