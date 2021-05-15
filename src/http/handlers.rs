use super::{
    errors::{BodyParsingError, GitError, SignatureError},
    SharedConfig,
};
use crate::{
    github::{Github, ReleaseAction},
    repo,
};
use bytes::Bytes;
use git2::Repository;
use ring::hmac;
use tracing::{debug, info};
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

    // Extract the repository information and reference
    let (repository, fetch_refspec, merge_refspec) = match body {
        Github::Ping { zen, hook_id } => {
            info!("received ping from hook {}: {}", hook_id, zen);
            return Ok(StatusCode::NO_CONTENT);
        }
        Github::Push {
            after,
            reference,
            repository,
        } => (repository, reference, Some(after)),
        Github::Release {
            action,
            repository,
            release,
        } => {
            // Only do stuff when released
            if action == ReleaseAction::Released {
                let tag_refspec = format!("refs/tags/{}", release.tag_name);
                (repository, tag_refspec, None)
            } else {
                return Ok(StatusCode::NO_CONTENT);
            }
        }
    };

    // Build the repository path
    let folder_name = repository.name.replace("/", "__");
    let path = config.server.repositories.join(folder_name);

    // Initialize the repository
    let repo = Repository::init(&path).unwrap();

    // Get the repository's remote to pull
    repo.remote_set_url("origin", &repository.clone_url)
        .unwrap();
    let mut remote = repo.find_remote("origin").unwrap();

    // Download the repository
    // TODO: support private repositories
    info!("pulling {} for {}", fetch_refspec, repository.name);
    let fetch_commit = repo::fetch(&repo, &[&fetch_refspec], &mut remote)
        .map_err(|e| reject::custom(GitError(e)))?;

    // Merge the fetched data
    info!(
        "merging into {}",
        fetch_commit.refname().unwrap_or(&fetch_refspec)
    );
    repo::merge(&repo, &fetch_refspec, fetch_commit).map_err(|e| reject::custom(GitError(e)))?;

    // Checkout the last pushed commit (if it is a push)
    if let Some(commit) = merge_refspec {
        info!("checking out commit {}", commit);
        repo::checkout(&repo, &commit).map_err(|e| reject::custom(GitError(e)))?;
    }

    Ok(StatusCode::NO_CONTENT)
}
