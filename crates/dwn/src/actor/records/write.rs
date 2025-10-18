use dwn_core::message::{Message, Version, descriptor::RecordsWriteBuilder, mime::Mime};
use reqwest::Url;
use xdid::core::did::Did;

use crate::Actor;

impl Actor {
    pub fn write(&self) -> ActorWriteBuilder<'_> {
        ActorWriteBuilder {
            actor: self,
            msg: RecordsWriteBuilder::default(),
            auth: true,
            sign: false,
            sync: true,
            target: None,
        }
    }
}

pub struct ActorWriteBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsWriteBuilder,
    auth: bool,
    sign: bool,
    sync: bool,
    target: Option<&'a Did>,
}

impl<'a> ActorWriteBuilder<'a> {
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

    pub fn protocol(mut self, protocol: String, version: Version, path: String) -> Self {
        self.msg.protocol = Some(protocol);
        self.msg.protocol_version = Some(version);
        self.msg.protocol_path = Some(path);
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

    /// Whether to sync the message with the actor's remote DWN after processing.
    /// Defaults to `true`.
    pub fn sync(mut self, value: bool) -> Self {
        self.sync = value;
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

        if self.sign {
            self.actor.sign(&mut msg)?;
        }
        if self.auth {
            self.actor.authorize(&mut msg)?;
        }

        Ok(msg)
    }

    /// Sends the message to a remote DWN.
    /// Returns the written record ID.
    pub async fn send(self, url: &Url) -> anyhow::Result<String> {
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;
        let id = msg.record_id.clone();

        actor.send(target, &msg, url).await?;

        Ok(id)
    }

    /// Processes the message with the actor's local DWN.
    /// Returns the written record ID.
    pub async fn process(self) -> anyhow::Result<String> {
        let sync = self.sync;
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;
        let id = msg.record_id.clone();

        if sync && actor.remote.is_some() {
            actor.send_remote(target, &msg).await?;
        }

        actor
            .dwn
            .process_message(target, msg)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to process message: {e}"))?;

        Ok(id)
    }
}
