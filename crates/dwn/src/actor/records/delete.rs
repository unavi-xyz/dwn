use dwn_core::message::descriptor::RecordsDeleteBuilder;

use crate::Actor;

impl Actor {
    pub fn delete(&self, record_id: String) -> ActorDeleteBuilder<'_> {
        ActorDeleteBuilder {
            actor: self,
            msg: RecordsDeleteBuilder::new(record_id),
            auth: true,
            sync: true,
        }
    }
}

pub struct ActorDeleteBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsDeleteBuilder,
    auth: bool,
    sync: bool,
}

impl ActorDeleteBuilder<'_> {
    /// Whether to authorize the message.
    /// Defaults to `true`.
    pub fn auth(mut self, value: bool) -> Self {
        self.auth = value;
        self
    }

    /// Whether to sync the message with the remote.
    /// Defaults to `true`.
    pub fn sync(mut self, value: bool) -> Self {
        self.sync = value;
        self
    }

    /// Processes the message with the actor's DWN.
    pub async fn process(self) -> anyhow::Result<()> {
        let mut msg = self.msg.build()?;

        if self.auth {
            self.actor.authorize(&mut msg)?;
        }

        if self.sync && self.actor.remote.is_some() {
            self.actor.send_remote(&self.actor.did, &msg).await?;
        }

        let _ = self
            .actor
            .dwn
            .process_message(&self.actor.did, msg)
            .await
            .map_err(|e| anyhow::anyhow!("failed to process message: {e}"))?;

        Ok(())
    }
}
