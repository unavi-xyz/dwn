use dwn_core::message::{Message, descriptor::RecordsDeleteBuilder};
use reqwest::Url;
use xdid::core::did::Did;

use crate::Actor;

impl Actor {
    pub fn delete(&self, record_id: String) -> ActorDeleteBuilder<'_> {
        ActorDeleteBuilder {
            actor: self,
            msg: RecordsDeleteBuilder::new(record_id),
            auth: true,
            sync: true,
            target: None,
        }
    }
}

pub struct ActorDeleteBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsDeleteBuilder,
    auth: bool,
    sync: bool,
    target: Option<&'a Did>,
}

impl<'a> ActorDeleteBuilder<'a> {
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

    /// Whether to sync the message with the remote.
    /// Defaults to `true`.
    pub fn sync(mut self, value: bool) -> Self {
        self.sync = value;
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
    pub async fn send_remote(self) -> anyhow::Result<()> {
        let url = self
            .actor
            .remote
            .as_ref()
            .ok_or(anyhow::anyhow!("no remote"))?;
        self.send(url).await
    }

    /// Sends the message to a remote DWN.
    pub async fn send(self, url: &Url) -> anyhow::Result<()> {
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;

        actor.send(target, &msg, url).await?;

        Ok(())
    }

    /// Processes the message with the actor's DWN.
    pub async fn process(self) -> anyhow::Result<()> {
        let sync = self.sync;
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;

        if sync && actor.remote.is_some() {
            actor.send_remote(&actor.did, &msg).await?;
        }

        let _ = actor
            .dwn
            .process_message(target, msg)
            .await
            .map_err(|e| anyhow::anyhow!("failed to process message: {e}"))?;

        Ok(())
    }
}
