use surrealdb::{
    sql::{Id, Table, Thing},
    Connection, Surreal,
};

use crate::message::{CidError, Message};

use self::model::{CreateEncodedMessage, GetEncodedMessage};

use super::MessageStore;

pub mod model;

#[derive(thiserror::Error, Debug)]
pub enum MessageStoreError {
    #[error("Message missing data")]
    MissingData,
    #[error("Failed to generate CID: {0}")]
    CidError(#[from] CidError),
    #[error("Failed to interact with SurrealDB: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("Failed to encode data: {0}")]
    DataEncodeError(#[from] libipld_core::error::SerdeError),
}

pub struct SurrealDB<T: Connection> {
    db: Surreal<T>,
}

impl<T: Connection> MessageStore for SurrealDB<T> {
    type Error = MessageStoreError;

    fn delete(&self, tenant: &str, cid: String) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn get(&self, tenant: &str, cid: String) -> Result<Message, Self::Error> {
        unimplemented!()
    }

    async fn put(&self, tenant: &str, message: Message) -> Result<libipld::Cid, Self::Error> {
        let data = message.data.as_ref().ok_or(Self::Error::MissingData)?;
        let encoded_data = data.encode()?;

        let block = message.cbor_block()?;
        let cid = block.cid();

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        self.db
            .create::<Option<GetEncodedMessage>>(id.clone())
            .content(CreateEncodedMessage {
                cid: cid.to_string(),
                encoded_data: Some(encoded_data),
                encoded_message: block.data().to_vec(),
                tenant: tenant.to_string(),
            })
            .await?;

        Ok(*cid)
    }
}
