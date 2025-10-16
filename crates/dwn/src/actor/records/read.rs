use anyhow::bail;
use dwn_core::{message::descriptor::RecordsReadBuilder, reply::Reply};

use crate::{Actor, records::RecordView};

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
