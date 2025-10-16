use anyhow::bail;
use base64::{DecodeError, Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dwn_core::{
    message::{Message, data::Data, descriptor::RecordsReadBuilder},
    reply::Reply,
};

use crate::Actor;

impl Actor {
    pub fn read(&self, record_id: String) -> ActorReadBuilder<'_> {
        ActorReadBuilder {
            actor: self,
            msg: RecordsReadBuilder::new(record_id),
            auth: true,
        }
    }
}

pub struct ActorReadBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsReadBuilder,
    auth: bool,
}

impl ActorReadBuilder<'_> {
    /// Whether to authorize the message.
    /// Defaults to `true`.
    pub fn auth(mut self, value: bool) -> Self {
        self.auth = value;
        self
    }

    /// Processes the message with the actor's DWN.
    pub async fn process(self) -> anyhow::Result<Option<RecordView>> {
        let mut msg = self.msg.build()?;

        if self.auth {
            self.actor.authorize(&mut msg)?;
        }

        let reply = self
            .actor
            .dwn
            .process_message(&self.actor.did, msg)
            .await
            .map_err(|e| anyhow::anyhow!("failed to process message: {e}"))?;

        match reply {
            Some(Reply::RecordsRead(read)) => match read.entry {
                Some(entry) => Ok(Some(RecordView::from_entry(entry)?)),
                None => Ok(None),
            },
            Some(other) => {
                bail!("got invalid reply from DWN: {other:?}")
            }
            None => {
                bail!("got no reply from DWN")
            }
        }
    }
}

pub struct RecordView {
    data: Option<Vec<u8>>,
    entry: Message,
}

impl RecordView {
    fn from_entry(mut entry: Message) -> Result<Self, DecodeError> {
        let data = match entry.data.take() {
            Some(Data::Base64(encoded)) => {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(encoded)?;
                Some(decoded)
            }
            Some(Data::Encrypted(_)) => todo!(),
            None => None,
        };

        Ok(Self { data, entry })
    }

    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }
    pub fn into_data(self) -> Option<Vec<u8>> {
        self.data
    }

    pub fn entry(&self) -> &Message {
        &self.entry
    }
}
