use async_channel::Sender;

mod message;
mod worker;

pub use message::Message;

/// Create a new deployment processor
pub fn create(num_workers: u32) -> Sender<Message> {
    // Create the channels
    let (tx, rx) = async_channel::unbounded();

    // Spawn the workers
    for _ in 0..num_workers {
        tokio::spawn(worker::worker(rx.clone()));
    }

    tx
}
