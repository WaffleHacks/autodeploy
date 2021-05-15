use super::Message;
use async_channel::Receiver;

/// Process incoming deployment workloads
pub async fn worker(receiver: Receiver<Message>) {
    // TODO: implement processor
}
