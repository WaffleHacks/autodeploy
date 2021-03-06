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
        after: String,
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

impl Github {
    /// Get the name of the webhook being used
    pub fn name<'a>(&self) -> &'a str {
        match self {
            Self::Ping { .. } => "ping",
            Self::Push { .. } => "push",
            Self::Release { .. } => "release",
        }
    }
}

/// Possible release actions that can be done
#[derive(Debug, Deserialize, PartialEq)]
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
    pub tag_name: String,
}

/// The repository information
#[derive(Clone, Debug, Deserialize)]
pub struct Repository {
    #[serde(rename = "full_name")]
    pub name: String,
    pub clone_url: String,
}
