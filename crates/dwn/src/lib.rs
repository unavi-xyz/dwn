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
    message::{Message, descriptor::Descriptor},
    reply::Reply,
    store::{DataStore, RecordStore},
};
use reqwest::StatusCode;
use tracing::debug;
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
            Descriptor::RecordsDelete(_) => {
                handlers::records::delete::handle(
                    self.data_store.as_ref(),
                    self.record_store.as_ref(),
                    target,
                    msg,
                )?;
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
}
