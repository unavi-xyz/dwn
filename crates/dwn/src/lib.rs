//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! The DWN spec is a work-in-progress and often out of date from other implementations,
//! so it is treated more as a loose guide rather than an absolute set of rules to follow.

use std::sync::Arc;

use dwn_core::{
    message::{Interface, Message, Method},
    reply::Reply,
    store::{DataStore, RecordStore},
};
use reqwest::{Client, StatusCode};
use tracing::debug;
use xdid::core::did::Did;

pub use dwn_core as core;

pub mod stores {
    #[cfg(feature = "native_db")]
    pub use dwn_native_db::*;
}

pub mod actor;
pub mod builders;
mod handlers;

#[derive(Clone)]
pub struct Dwn {
    pub data_store: Arc<dyn DataStore>,
    pub record_store: Arc<dyn RecordStore>,
    /// URL of the remote DWN to sync with.
    pub remote: Option<String>,
    client: Client,
    queue: Vec<(Did, Message)>,
}

impl<T: DataStore + RecordStore + Clone + 'static> From<T> for Dwn {
    fn from(value: T) -> Self {
        Self {
            data_store: Arc::new(value.clone()),
            record_store: Arc::new(value),
            remote: None,
            client: Client::new(),
            queue: Vec::new(),
        }
    }
}

impl Dwn {
    pub async fn process_message(
        &mut self,
        target: &Did,
        msg: Message,
    ) -> Result<Option<Reply>, StatusCode> {
        debug!(
            "Processing {} {}",
            msg.descriptor.interface.to_string(),
            msg.descriptor.method.to_string()
        );

        if let Err(e) = handlers::validation::validate_message(target, &msg).await {
            debug!("Failed to validate message: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        };

        let res = match &msg.descriptor.interface {
            Interface::Records => match &msg.descriptor.method {
                Method::Read => {
                    handlers::records::read::handle(self.record_store.as_ref(), target, msg)
                        .map(|v| Some(Reply::RecordsRead(Box::new(v))))?
                }
                Method::Query => {
                    handlers::records::query::handle(self.record_store.as_ref(), target, msg)
                        .await
                        .map(|v| Some(Reply::RecordsQuery(v)))?
                }
                Method::Write => {
                    handlers::records::write::handle(
                        self.record_store.as_ref(),
                        target,
                        msg.clone(),
                    )
                    .await?;

                    if self.remote.is_some() {
                        self.queue.push((target.clone(), msg));
                    }

                    None
                }
                Method::Subscribe => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                Method::Delete => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                _ => return Err(StatusCode::BAD_REQUEST),
            },
            Interface::Protocols => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            Interface::Permissions => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        Ok(res)
    }

    /// Sends a message to a remote DWN.
    pub async fn send(
        &self,
        target: &Did,
        msg: &Message,
        url: &str,
    ) -> Result<Option<Reply>, reqwest::Error> {
        send_remote(&self.client, target, msg, url).await
    }

    /// Syncs newly processed messages with the remote DWN.
    pub async fn sync(&mut self) -> Result<(), reqwest::Error> {
        let Some(remote) = self.remote.as_deref() else {
            return Ok(());
        };

        while !self.queue.is_empty() {
            // If send fails, we still remove the message from the queue.
            // Should we instead attempt to handle the error and send again?
            let (target, msg) = self.queue.remove(0);
            send_remote(&self.client, &target, &msg, remote).await?;
        }

        Ok(())
    }
}

async fn send_remote(
    client: &Client,
    target: &Did,
    msg: &Message,
    url: &str,
) -> Result<Option<Reply>, reqwest::Error> {
    let url = format!("{}/{}", url, target);
    let req = client.put(url).json(msg).build()?;
    let res = client.execute(req).await?;
    let reply = res.json::<Option<Reply>>().await?;
    Ok(reply)
}
