use dwn_core::message::{
    Version,
    descriptor::{ProtocolDefinition, ProtocolsConfigureBuilder},
};

use crate::Actor;

impl Actor {
    pub fn configure_protocol(
        &self,
        version: Version,
        definition: ProtocolDefinition,
    ) -> ActorConfigureProtocolBuilder<'_> {
        ActorConfigureProtocolBuilder {
            actor: self,
            msg: ProtocolsConfigureBuilder::new(version, definition),
            auth: true,
            sync: true,
        }
    }
}

pub struct ActorConfigureProtocolBuilder<'a> {
    actor: &'a Actor,
    msg: ProtocolsConfigureBuilder,
    auth: bool,
    sync: bool,
}

impl ActorConfigureProtocolBuilder<'_> {
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
