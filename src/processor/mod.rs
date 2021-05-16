use async_channel::Sender;
use tracing::info;

mod config;
mod message;
mod worker;

pub use message::Message;

/// Create a new deployment processor
pub fn create(num_workers: u32) -> Sender<Message> {
    // Create the channels
    let (tx, rx) = async_channel::unbounded();

    info!(count = num_workers, "spawning deployment workers");

    // Spawn the workers
    for id in 0..num_workers {
        tokio::spawn(worker::worker(id, rx.clone()));
    }

    tx
}
