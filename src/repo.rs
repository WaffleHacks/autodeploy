use git2::{
    build::CheckoutBuilder, AnnotatedCommit, AutotagOption, FetchOptions, Oid, Reference, Remote,
    RemoteCallbacks, Repository, ResetType,
};
use tracing::{debug, error, info};

type Result<T> = std::result::Result<T, git2::Error>;

/// Checkout the specified commit by SHA1 hash
pub fn checkout(repo: &Repository, hash: &str) -> Result<()> {
    // Find the commit
    let oid = Oid::from_str(hash)?;
    let commit = repo.find_commit(oid)?;

    // Reset the current working tree to the desired commit
    repo.reset(
        commit.as_object(),
        ResetType::Hard,
        Some(CheckoutBuilder::default().force()),
    )?;

    Ok(())
}

/// Fetch all the data in the given refspec
pub fn fetch<'r>(
    repo: &'r Repository,
    refs: &[&str],
    remote: &'r mut Remote,
) -> Result<AnnotatedCommit<'r>> {
    // Log transfer progress
    let mut callback = RemoteCallbacks::new();
    callback.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            debug!(
                "resolving deltas {}/{}",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            debug!(
                "received {}/{} objects ({}) in {} bytes",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        true
    });

    // Fetch from the remote
    // Always fetch tags
    remote.fetch(
        refs,
        Some(
            FetchOptions::new()
                .remote_callbacks(callback)
                .download_tags(AutotagOption::All),
        ),
        None,
    )?;

    // Log the stats of the fetch
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        info!(
            "received {}/{} objects in {} bytes (used {} local objects",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        info!(
            "received {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    repo.reference_to_annotated_commit(&fetch_head)
}

/// Merge the pulled branch and the current history
/// Supports normal merge and fast-forwarding, but will not try to resolve conflicts.
pub fn merge(repo: &Repository, refname: &str, fetch_commit: AnnotatedCommit) -> Result<()> {
    // Run a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // Do the appropriate merge (fast-forward/normal/none)
    if analysis.0.is_fast_forward() {
        info!("merging with fast-forward");

        match repo.find_reference(&refname) {
            Ok(mut r) => fast_forward(repo, &mut r, &fetch_commit)?,
            Err(_) => {
                // Set reference to commit directly
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("setting {} to {}", refname, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;

                // Checkout the head
                repo.checkout_head(Some(
                    CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        }
    } else if analysis.0.is_normal() {
        info!("merging normally");

        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        info!("no merge necessary");
    }

    Ok(())
}

/// Perform a fast forward merge
fn fast_forward(
    repo: &Repository,
    local_branch: &mut Reference,
    remote_commit: &AnnotatedCommit,
) -> Result<()> {
    // Get the name of the branch
    let name = match local_branch.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(local_branch.name_bytes()).to_string(),
    };

    // Re-target the current branch
    local_branch.set_target(
        remote_commit.id(),
        &format!(
            "Fast-forward: setting {} to id {}",
            name,
            remote_commit.id()
        ),
    )?;

    // Set the head
    repo.set_head(&name)?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;

    info!(
        "successfully fast-forwarded {} to {}",
        name,
        remote_commit.id()
    );

    Ok(())
}

/// Perform a normal merge
fn normal_merge(
    repo: &Repository,
    local: &AnnotatedCommit,
    remote: &AnnotatedCommit,
) -> Result<()> {
    // Find the common ancestor between the two commits
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;

    // Merge the two trees
    let mut index = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    // Don't attempt to resolve conflicts
    if index.has_conflicts() {
        error!("merge conflicts detected, cannot resolve automatically");
        repo.checkout_index(Some(&mut index), None)?;
        return Ok(());
    }

    // Finalize the merge
    let result_tree = repo.find_tree(index.write_tree_to(repo)?)?;
    let signature = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Merge: {} into {}", remote.id(), local.id()),
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;

    info!("successfully merged from {} to {}", remote.id(), local.id());

    Ok(())
}
