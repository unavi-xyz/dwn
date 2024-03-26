use reqwest::Client;
use thiserror::Error;
use tokio::sync::mpsc::{self};

use crate::{
    handlers::Response,
    message::{Message, Request},
};

/// Sends new messages to a remote DWN.
pub struct RemoteSync {
    client: Client,
    message_recv: mpsc::Receiver<Message>,
    pub(crate) message_send: mpsc::Sender<Message>,
    remote_url: String,
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl RemoteSync {
    pub fn new(remote_url: String) -> Self {
        let (message_send, message_recv) = mpsc::channel(100);

        Self {
            client: Client::new(),
            message_recv,
            message_send,
            remote_url,
        }
    }

    pub async fn sync(&mut self) -> Result<Option<Response>, SyncError> {
        let mut messages = Vec::new();

        while let Ok(message) = self.message_recv.try_recv() {
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
