//! Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).
//!
//! The DWN spec is a work-in-progress and often out of date from other implementations,
//! so it is treated more as a loose guide rather than an absolute set of rules to follow.
//!
//! # Example
//!
//! ```
//! use dwn::{
//!     core::{message::{descriptor::{RecordsReadBuilder, RecordsWriteBuilder}, mime::TEXT_PLAIN}, reply::Reply},
//!     stores::NativeDbStore,
//!     Actor,
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
//!     // Create an actor to sign messages on behalf of our DID.
//!     let mut actor = Actor::new(did.clone());
//!     actor.auth_key = Some(key.clone().into());
//!     actor.sign_key = Some(key.into());
//!    
//!     // Prepare to write a new record to the DWN.
//!     let mut msg = RecordsWriteBuilder::default()
//!         .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_vec())
//!         .published(true)
//!         .build()
//!         .unwrap();
//!    
//!     let record_id = msg.record_id.clone();
//!    
//!     // Authorize the message using the actor.
//!     actor.authorize(&mut msg).unwrap();
//!    
//!     // Process the message at our DID's DWN.
//!     dwn.process_message(&did, msg.clone()).await.unwrap();
//!    
//!     // We can now read the record using its ID.
//!     let read = RecordsReadBuilder::new(record_id.clone())
//!         .build()
//!         .unwrap();
//!    
//!     let reply = dwn.process_message(&did, read).await.unwrap();
//!
//!     let found = match reply {
//!         Some(Reply::RecordsRead(r)) => r.entry.unwrap(),
//!         _ => panic!("invalid reply"),
//!     };
//!
//!     assert_eq!(found, msg);
//! }
//! ```

use std::sync::Arc;

use dwn_core::{
    message::{cid::CidGenerationError, descriptor::Descriptor, Message},
    reply::Reply,
    store::{DataStore, RecordStore, StoreError},
};
use reqwest::{Client, StatusCode};
use thiserror::Error;
use tracing::{debug, warn};
use xdid::core::did::Did;

pub use dwn_core as core;

pub mod stores {
    #[cfg(feature = "native_db")]
    pub use dwn_native_db::*;
}

mod actor;
mod handlers;

pub use actor::*;

#[derive(Clone)]
pub struct Dwn {
    pub data_store: Arc<dyn DataStore>,
    pub record_store: Arc<dyn RecordStore>,
    /// URL of the remote DWN to sync with.
    pub remote: Option<String>,
    client: Client,
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
        }
    }

    pub async fn process_message(
        &mut self,
        target: &Did,
        msg: Message,
    ) -> Result<Option<Reply>, StatusCode> {
        if let Err(e) = handlers::validation::validate_message(target, &msg).await {
            debug!("Failed to validate message: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        };

        let res = match &msg.descriptor {
            Descriptor::RecordsQuery(_) => {
                handlers::records::query::handle(self.record_store.as_ref(), target, msg)
                    .await
                    .map(|v| Some(Reply::RecordsQuery(v)))?
            }
            Descriptor::RecordsRead(_) => handlers::records::read::handle(
                self.data_store.as_ref(),
                self.record_store.as_ref(),
                target,
                msg,
            )
            .map(|v| Some(Reply::RecordsRead(Box::new(v))))?,
            Descriptor::RecordsSync(_) => handlers::records::sync::handle(
                self.data_store.as_ref(),
                self.record_store.as_ref(),
                target,
                msg,
            )
            .map(|v| Some(Reply::RecordsSync(Box::new(v))))?,
            Descriptor::RecordsWrite(_) => {
                handlers::records::write::handle(
                    self.data_store.as_ref(),
                    self.record_store.as_ref(),
                    target,
                    msg,
                )
                .await?;
                None
            }
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

    /// Syncs with the remote DWN.
    /// If an actor is provided, it will be used to authorize the sync.
    pub async fn sync(&mut self, target: &Did, actor: Option<&Actor>) -> Result<(), SyncError> {
        let Some(remote) = &self.remote.clone() else {
            return Ok(());
        };

        let descriptor = Descriptor::RecordsSync(Box::new(
            self.record_store.prepare_sync(target, actor.is_some())?,
        ));

        let mut msg = Message {
            record_id: descriptor.compute_entry_id()?,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        };

        if let Some(actor) = actor {
            actor.authorize(&mut msg)?;
        }

        let reply = match self.send(target, &msg, remote).await? {
            Some(Reply::RecordsSync(r)) => r,
            v => {
                debug!("Invalid reply: {:?}", v);
                return Err(SyncError::InvalidReply);
            }
        };

        // Process new records.
        for record in reply.remote_only {
            if let Err(e) = self.process_message(target, record.initial_entry).await {
                warn!("Failed to process message during DWN sync: {}", e);
                continue;
            };

            if let Err(e) = self.process_message(target, record.latest_entry).await {
                warn!("Failed to process message during DWN sync: {}", e);
            };
        }

        // Process conflicting entries.
        for entry in reply.conflict {
            if let Err(e) = self.process_message(target, entry).await {
                warn!("Failed to process message during DWN sync: {}", e);
            };
        }

        // Send local records to remote.
        for record_id in reply.local_only {
            let Some(record) =
                self.record_store
                    .read(self.data_store.as_ref(), target, &record_id)?
            else {
                continue;
            };

            self.send(target, &record.initial_entry, remote).await?;

            if record.latest_entry.descriptor.compute_entry_id()? != record.initial_entry.record_id
            {
                self.send(target, &record.latest_entry, remote).await?;
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SyncError {
    #[error(transparent)]
    CidGeneration(#[from] CidGenerationError),
    #[error("invalid reply from remote")]
    InvalidReply,
    #[error(transparent)]
    RecordStore(#[from] StoreError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to authorize message: {0}")]
    Sign(#[from] SignError),
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
