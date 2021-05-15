use async_channel::Sender;
use std::path::PathBuf;

/// The message to be sent from the webhook handler
/// to the deployment processor containing the necessary
/// information to deploy the repository.
#[derive(Debug)]
pub struct Message {
    path: PathBuf,
    repository: String,
}

impl Message {
    /// Send a new message
    pub async fn send(sender: Sender<Self>, path: PathBuf, repository: String) {
        sender.send(Self { path, repository }).await.unwrap();
    }
}
