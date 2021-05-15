use super::{
    errors::{SignatureError, UndeployableError},
    SharedConfig,
};
use crate::github::Github;
use ring::hmac;
use tracing::{debug, warn};
use warp::{reject, Rejection};

type Result = std::result::Result<(), Rejection>;

/// Ensure that the signature from github is valid
pub(crate) fn valid_signature(raw_body: &[u8], raw_signature: String, secret: &[u8]) -> Result {
    // Remove the sha256 prefix from the hash
    let signature_hex = raw_signature
        .strip_prefix("sha256=")
        .ok_or_else(|| reject::custom(SignatureError))?;
    let signature = hex::decode(signature_hex).map_err(|_| reject::custom(SignatureError))?;

    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);

    // Display the expected signature in debug builds
    #[cfg(debug_assertions)]
    debug!(
        "signature validation: expected: {}, got: {}",
        hex::encode(hmac::sign(&key, raw_body).as_ref()),
        signature_hex
    );

    // Check if the signature is valid
    hmac::verify(&key, raw_body, &signature).map_err(|_| reject::custom(SignatureError))
}

/// Check that the received repository is allowed to be deployed
pub(crate) fn deployable(config: &SharedConfig, body: &Github) -> Result {
    // Default to allow
    if config.events.is_empty() {
        return Ok(());
    }

    // Get the repository name and branch (if push)
    let (name, branch) = match body {
        Github::Ping { .. } => return Ok(()), // Pings are always allowed
        Github::Push {
            repository,
            reference,
            ..
        } => {
            // Get the branch it was pushed to
            let pushed_branch = reference.trim_start_matches("refs/heads/");

            (&repository.name, Some(pushed_branch))
        }
        Github::Release { repository, .. } => (&repository.name, None),
    };

    // Check the branch and repository name are allowed
    for event in &config.events {
        if event.matches(name, branch) {
            return Ok(());
        }
    }

    if let Some(branch) = branch {
        warn!(
            "attempt to deploy {}#{} on {} was blocked",
            name,
            branch,
            body.name()
        );
    } else {
        warn!("attempt to deploy {} on {} was blocked", name, body.name());
    }
    Err(reject::custom(UndeployableError))
}
