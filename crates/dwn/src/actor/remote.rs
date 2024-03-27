use tokio::sync::{mpsc, RwLock};

use crate::message::Message;

pub struct Remote {
    pub(crate) receiver: RwLock<mpsc::Receiver<Message>>,
    pub(crate) sender: mpsc::Sender<Message>,
    url: String,
}

impl Remote {
    /// Create a new remote with a message queue size of 100.
    pub fn new(url: String) -> Self {
        Self::new_with_capacity(url, 100)
    }

    /// Create a new remote with a message queue size of `capacity`.
    pub fn new_with_capacity(url: String, capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        Self {
            receiver: RwLock::new(receiver),
            url,
            sender,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
