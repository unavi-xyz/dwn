use reqwest::Client;
use tokio::sync::{mpsc, Mutex};

use crate::{
    handlers::Response,
    message::{Message, Request},
};

pub struct Remote {
    client: Client,
    pub(crate) sender: mpsc::Sender<Message>,
    pub(crate) url: String,
    receiver: Mutex<mpsc::Receiver<Message>>,
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
            client: Client::new(),
            receiver: Mutex::new(receiver),
            url,
            sender,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Send messages to the remote DWN.
    pub async fn send(&self, messages: Vec<Message>) -> Result<Response, reqwest::Error> {
        self.client
            .post(&self.url)
            .json(&Request { messages })
            .send()
            .await?
            .json::<Response>()
            .await
    }

    /// Sync with the remote DWN.
    /// This will only sync records we have locally, it will not pull new records from the remote.
    pub async fn sync(&self) -> Result<(), reqwest::Error> {
        self.push().await?;
        self.pull().await?;
        Ok(())
    }

    /// Clear the message queue by sending all messages to the remote server.
    async fn push(&self) -> Result<(), reqwest::Error> {
        let mut messages = Vec::new();

        while let Ok(message) = self.receiver.lock().await.try_recv() {
            messages.push(message);
        }

        if !messages.is_empty() {
            let _ = self.send(messages).await;
        }

        Ok(())
    }

    /// Pull all locally stored records from the remote server.
    async fn pull(&self) -> Result<(), reqwest::Error> {
        Ok(())
    }
}
