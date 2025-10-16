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
//!     let dwn = Dwn::from(store);
//!    
//!     // Create a new did:key.
//!     let key = P256KeyPair::generate();
//!     let did = key.public().to_did();
//!    
//!     // Create an actor to sign messages on behalf of our DID.
//!     let mut actor = Actor::new(did, dwn);
//!     actor.auth_key = Some(key.clone().into());
//!     actor.sign_key = Some(key.into());
//!    
//!     // Write a new record to the DWN.
//!     let data = "Hello, world!".as_bytes().to_vec();
//!
//!     let record_id = actor.write()
//!         .data(TEXT_PLAIN, data.clone())
//!         .published(true)
//!         .process()
//!         .await
//!         .unwrap();
//!
//!     // We can now read the record using its ID.
//!     let found = actor.read(record_id.clone())
//!         .process()
//!         .await
//!         .unwrap()
//!         .unwrap();
//!
//!    assert!(found.entry().record_id, record_id);
//!    assert!(found.data().unwrap(), data);
//! }
//! ```

use std::sync::Arc;

use dwn_core::{
    message::{Message, cid::CidGenerationError, descriptor::Descriptor},
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
    /// Whether to automatically send writes to the remote.
    pub sync_writes: bool,
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
            sync_writes: true,
            client: Client::new(),
        }
    }

    pub async fn process_message(
        &self,
        target: &Did,
        msg: Message,
    ) -> Result<Option<Reply>, StatusCode> {
        if let Err(e) = handlers::validation::validate_message(target, &msg).await {
            debug!("Failed to validate message: {:?}", e);
            return Err(StatusCode::BAD_REQUEST);
        };

        let res = match &msg.descriptor {
            Descriptor::ProtocolsConfigure(_) => {
                // TODO
                None
            }
            Descriptor::ProtocolsQuery(_) => {
                // TODO
                None
            }
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
                if self.sync_writes
                    && let Some(remote) = self.remote.clone()
                {
                    let client = self.client.clone();
                    let msg = msg.clone();
                    let target = target.clone();
                    tokio::spawn(async move {
                        if let Err(e) = send_remote(&client, &target, &msg, &remote).await {
                            warn!("Remote write: {e:?}");
                        };
                    });
                }

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

    /// Sends a message to the remote DWN.
    pub async fn send_remote(
        &self,
        target: &Did,
        msg: &Message,
    ) -> Result<Option<Reply>, SyncError> {
        let Some(remote) = self.remote.as_deref() else {
            return Err(SyncError::NoRemote);
        };
        let reply = send_remote(&self.client, target, msg, remote).await?;
        Ok(reply)
    }

    /// Full sync with the remote DWN.
    /// If an actor is provided, it will be used to authorize the sync.
    pub async fn sync(&mut self, target: &Did, actor: Option<&Actor>) -> Result<(), SyncError> {
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

        let reply = match self.send_remote(target, &msg).await? {
            Some(Reply::RecordsSync(r)) => r,
            r => {
                debug!("Invalid reply: {r:?}");
                return Err(SyncError::InvalidReply);
            }
        };

        // Process new records.
        for record in reply.remote_only {
            if let Err(e) = self.process_message(target, record.initial_entry).await {
                warn!("Failed to process message during DWN sync: {e}");
                continue;
            };

            if let Err(e) = self.process_message(target, record.latest_entry).await {
                warn!("Failed to process message during DWN sync: {e}");
            };
        }

        // Process conflicting entries.
        for entry in reply.conflict {
            if let Err(e) = self.process_message(target, entry).await {
                warn!("Failed to process message during DWN sync: {e}");
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

            self.send_remote(target, &record.initial_entry).await?;

            if record.latest_entry.descriptor.compute_entry_id()? != record.initial_entry.record_id
            {
                self.send_remote(target, &record.latest_entry).await?;
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
    #[error("no remote url set")]
    NoRemote,
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
