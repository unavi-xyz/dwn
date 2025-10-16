use dwn_core::message::{Version, descriptor::RecordsWriteBuilder, mime::Mime};

use crate::Actor;

impl Actor {
    pub fn write(&self) -> ActorWriteBuilder<'_> {
        ActorWriteBuilder {
            actor: self,
            msg: RecordsWriteBuilder::default(),
            auth: true,
            sign: false,
            sync: true,
        }
    }
}

pub struct ActorWriteBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsWriteBuilder,
    auth: bool,
    sign: bool,
    sync: bool,
}

impl ActorWriteBuilder<'_> {
    pub fn record_id(mut self, value: String) -> Self {
        self.msg.record_id = Some(value);
        self
    }

    pub fn context_id(mut self, value: String) -> Self {
        self.msg.context_id = Some(value);
        self
    }

    pub fn data(mut self, format: Mime, data: Vec<u8>) -> Self {
        self.msg.data_format = Some(format);
        self.msg.data = Some(data);
        self
    }

    pub fn schema(mut self, value: String) -> Self {
        self.msg.schema = Some(value);
        self
    }

    pub fn protocol(mut self, protocol: String, version: Version) -> Self {
        self.msg.protocol = Some(protocol);
        self.msg.protocol_version = Some(version);
        self
    }

    pub fn published(mut self, value: bool) -> Self {
        self.msg.published = Some(value);
        self
    }

    /// Whether to authorize the message.
    /// Defaults to `true`.
    pub fn auth(mut self, value: bool) -> Self {
        self.auth = value;
        self
    }

    /// Whether to sign the message data.
    /// Defaults to `false`.
    pub fn sign(mut self, value: bool) -> Self {
        self.sign = value;
        self
    }

    /// Whether to sync the message with the remote.
    /// Defaults to `true`.
    pub fn sync(mut self, value: bool) -> Self {
        self.sync = value;
        self
    }

    /// Processes the message with the actor's DWN.
    /// Returns the written record ID.
    pub async fn process(self) -> anyhow::Result<String> {
        let mut msg = self.msg.build()?;

        if self.sign {
            self.actor.sign(&mut msg)?;
        }
        if self.auth {
            self.actor.authorize(&mut msg)?;
        }

        let id = msg.record_id.clone();

        if self.sync && self.actor.remote.is_some() {
            self.actor.send_remote(&msg).await?;
        }

        self.actor
            .dwn
            .process_message(&self.actor.did, msg)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to process message: {e}"))?;

        Ok(id)
    }
}
