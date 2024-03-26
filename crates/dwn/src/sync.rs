use reqwest::Client;
use tokio::sync::mpsc::{self};

use crate::{
    handlers::Response,
    message::{Message, Request},
};

/// Sends new messages to a remote DWN.
pub struct RemoteSync {
    client: Client,
    pub(crate) sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    remote_url: String,
}

impl RemoteSync {
    pub fn new(remote_url: String) -> Self {
        let (sender, receiver) = mpsc::channel(100);

        Self {
            client: Client::new(),
            receiver,
            remote_url,
            sender,
        }
    }

    pub async fn sync(&mut self) -> Result<Option<Response>, reqwest::Error> {
        let mut messages = Vec::new();

        while let Ok(message) = self.receiver.try_recv() {
            messages.push(message);
        }

        if messages.is_empty() {
            return Ok(None);
        }

        let response = self
            .client
            .post(&self.remote_url)
            .json(&Request { messages })
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(Some(response))
    }
}
