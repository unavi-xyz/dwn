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
    pub fn new(url: String) -> Self {
        let (sender, receiver) = mpsc::channel(100);

        Self {
            client: Client::new(),
            receiver: Mutex::new(receiver),
            url,
            sender,
        }
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
    pub async fn sync(&self) -> Result<Option<Response>, reqwest::Error> {
        self.push().await
    }

    /// Clear the message queue by sending all messages to the remote server.
    async fn push(&self) -> Result<Option<Response>, reqwest::Error> {
        let mut messages = Vec::new();

        while let Ok(message) = self.receiver.lock().await.try_recv() {
            messages.push(message);
        }

        if messages.is_empty() {
            return Ok(None);
        }

        self.send(messages).await.map(Some)
    }

    /// Pull all locally stored records from the remote server.
    async fn pull(&mut self) -> Result<(), reqwest::Error> {
        Ok(())
    }
}
