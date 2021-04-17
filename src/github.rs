use serde::Deserialize;

/// The overarching webhook types
#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum Github {
    Ping {
        zen: String,
        hook_id: i64,
    },
    Push {
        #[serde(rename = "ref")]
        reference: String,
        repository: Repository,
    },
    Release {
        action: ReleaseAction,
        repository: Repository,
        release: Release,
    },
}

/// Possible release actions that can be done
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseAction {
    Published,
    Unpublished,
    Created,
    Edited,
    Deleted,
    PreReleased,
    Released,
}

/// Information about a release
#[derive(Debug, Deserialize)]
pub struct Release {
    tag_name: String,
}

/// The repository information
#[derive(Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "full_name")]
    name: String,
    clone_url: String,
}
