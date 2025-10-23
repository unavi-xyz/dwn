use anyhow::bail;
use dwn_core::{
    message::{Message, descriptor::RecordsReadBuilder},
    reply::Reply,
};
use reqwest::Url;
use xdid::core::did::Did;

use crate::{Actor, records::RecordView};

impl Actor {
    pub fn read(&self, record_id: String) -> ActorReadBuilder<'_> {
        ActorReadBuilder {
            actor: self,
            msg: RecordsReadBuilder::new(record_id),
            auth: true,
            target: None,
        }
    }
}

pub struct ActorReadBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsReadBuilder,
    auth: bool,
    target: Option<&'a Did>,
}

impl<'a> ActorReadBuilder<'a> {
    /// Whether to authorize the message.
    /// Defaults to `true`.
    pub fn auth(mut self, value: bool) -> Self {
        self.auth = value;
        self
    }

    /// Sets the target DID for DWN processing.
    /// Defaults to the actor's own DID.
    pub fn target(mut self, value: &'a Did) -> Self {
        self.target = Some(value);
        self
    }

    fn build(self) -> anyhow::Result<Message> {
        let mut msg = self.msg.build()?;

        if self.auth {
            self.actor.authorize(&mut msg)?;
        }

        Ok(msg)
    }

    /// Sends the message to the actor's remote DWN.
    pub async fn send_remote(self) -> anyhow::Result<Option<RecordView>> {
        let url = self
            .actor
            .remote
            .as_ref()
            .ok_or(anyhow::anyhow!("no remote"))?;
        self.send(url).await
    }

    /// Sends the message to a remote DWN.
    pub async fn send(self, url: &Url) -> anyhow::Result<Option<RecordView>> {
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;

        let reply = actor.send(target, &msg, url).await?;

        parse_reply(reply)
    }

    /// Processes the message with the actor's local DWN.
    pub async fn process(self) -> anyhow::Result<Option<RecordView>> {
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;

        let reply = actor
            .dwn
            .process_message(target, msg)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to process message: {e}"))?;

        parse_reply(reply)
    }
}

fn parse_reply(reply: Option<Reply>) -> anyhow::Result<Option<RecordView>> {
    match reply {
        Some(Reply::RecordsRead(read)) => Ok(read.entry.map(RecordView::from_entry).transpose()?),
        Some(other) => {
            bail!("got invalid reply from DWN: {other:?}")
        }
        None => {
            bail!("got no reply from DWN")
        }
    }
}
