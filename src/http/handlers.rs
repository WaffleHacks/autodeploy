use super::{
    access,
    errors::{BodyParsingError, GitError},
    SharedConfig,
};
use crate::{
    github::{Github, ReleaseAction},
    processor::Message,
    repo,
};
use async_channel::Sender;
use bytes::Bytes;
use git2::Repository;
use std::path::Path;
use tracing::info;
use warp::{http::StatusCode, reject, Rejection, Reply};

/// Handle receiving webhooks from GitHub
pub async fn hook(
    raw_body: Bytes,
    raw_signature: String,
    config: SharedConfig,
    sender: Sender<Message>,
) -> Result<impl Reply, Rejection> {
    // Ensure the signature is valid
    access::valid_signature(&raw_body, raw_signature, config.server.secret.as_bytes())?;

    // Attempt to parse the body
    let body: Github =
        serde_json::from_slice(&raw_body).map_err(|_| reject::custom(BodyParsingError))?;
    info!("got new {} hook", body.name());

    // Ensure the repository is allowed to be deployed
    access::deployable(&config, &body)?;

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

    // Update the repository in a separate thread
    let arg_path = path.clone();
    let arg_repo = repository.clone();
    tokio::task::spawn_blocking(move || {
        update_repo(
            &arg_path,
            &arg_repo.name,
            &arg_repo.clone_url,
            fetch_refspec,
            merge_refspec,
        )
    })
    .await
    .unwrap()
    .map_err(|e| reject::custom(GitError(e)))?;

    // Queue the repository for processing
    Message::send(sender, path, repository.name).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Update the local copy of the repository
fn update_repo(
    path: &Path,
    name: &str,
    clone_url: &str,
    fetch_refspec: String,
    merge_refspec: Option<String>,
) -> Result<(), git2::Error> {
    // Initialize the repository
    let repo = Repository::init(path)?;

    // Get the repository's remote to pull
    repo.remote_set_url("origin", clone_url)?;
    let mut remote = repo.find_remote("origin").unwrap();

    // Download the repository
    // TODO: support private repositories
    info!("pulling {} for {}", fetch_refspec, name);
    let fetch_commit = repo::fetch(&repo, &[&fetch_refspec], &mut remote)?;

    // Merge the fetched data
    info!(
        "merging into {}",
        fetch_commit.refname().unwrap_or(&fetch_refspec)
    );
    repo::merge(&repo, &fetch_refspec, fetch_commit)?;

    // Checkout the last pushed commit (if it is a push)
    if let Some(commit) = merge_refspec {
        info!("checking out commit {}", commit);
        repo::checkout(&repo, &commit)?;
    }

    Ok(())
}
