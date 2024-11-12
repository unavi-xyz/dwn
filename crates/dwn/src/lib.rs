//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! The DWN spec is a work-in-progress and often out of date from other implementations,
//! so it is treated more as a loose guide rather than an absolute set of rules to follow.
//!
//! # Example
//!
//! ```
//! use dwn::{
//!     actor::Actor,
//!     builders::records::{read::RecordsReadBuilder, write::RecordsWriteBuilder},
//!     core::{message::mime::TEXT_PLAIN, reply::Reply},
//!     stores::NativeDbStore,
//!     Dwn
//! };
//! use xdid::methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a local in-memory DWN.
//!     let store = NativeDbStore::new_in_memory().unwrap();
//!     let mut dwn = Dwn::from(store);
//!    
//!     // Create a new did:key.
//!     let key = P256KeyPair::generate();
//!     let did = key.public().to_did();
//!    
//!     // Create an actor to sign messages on behalf of the DID.
//!     let mut actor = Actor::new(did.clone());
//!     actor.auth_key = Some(key.clone().into());
//!     actor.sign_key = Some(key.into());
//!    
//!     // Prepare to write a new record to the DWN.
//!     let mut msg = RecordsWriteBuilder::default()
//!         .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_owned())
//!         .published(true)
//!         .build()
//!         .unwrap();
//!    
//!     let record_id = msg.record_id.clone();
//!    
//!     // Authorize the message using our actor.
//!     actor.authorize(&mut msg).unwrap();
//!    
//!     // Process the write.
//!     dwn.process_message(&did, msg).await.unwrap();
//!    
//!     // We can now read the record using its ID.
//!     let msg = RecordsReadBuilder::new(record_id.clone())
//!         .build()
//!         .unwrap();
//!    
//!     let reply = dwn.process_message(&did, msg).await.unwrap();
//!
//!     let record = match reply {
//!         Some(Reply::RecordsRead(r)) => r.entry.unwrap(),
//!         _ => panic!("invalid reply"),
//!     };
//!
//!     assert_eq!(record.record_id, record_id);
//! }
//! ```

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
        Self::new(Arc::new(value.clone()), Arc::new(value))
    }
}

impl Dwn {
    pub fn new(data_store: Arc<dyn DataStore>, record_store: Arc<dyn RecordStore>) -> Self {
        Self {
            data_store,
            record_store,
            remote: None,
            client: Client::new(),
            queue: Vec::new(),
        }
    }

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
