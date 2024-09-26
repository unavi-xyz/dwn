use semver::Version;

use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    message::{
        descriptor::protocols::{ProtocolDefinition, ProtocolsConfigure},
        Message,
    },
    reply::{MessageReply, StatusReply},
};

pub struct ProtocolsConfigureBuilder<'a> {
    actor: &'a Actor,
    authorized: bool,
    definition: Option<ProtocolDefinition>,
    last_configuration: Option<String>,
    protocol_version: Version,
    target: Option<String>,
}

impl<'a> MessageBuilder for ProtocolsConfigureBuilder<'a> {
    fn get_actor(&self) -> &Actor {
        self.actor
    }

    fn get_authorized(&self) -> bool {
        self.authorized
    }
    fn authorized(mut self, authorized: bool) -> Self {
        self.authorized = authorized;
        self
    }

    fn get_target(&self) -> Option<String> {
        self.target.clone()
    }
    fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    fn create_message(&mut self) -> Result<Message, PrepareError> {
        let mut configure = ProtocolsConfigure::default();
        configure.definition.clone_from(&self.definition);
        configure
            .last_configuration
            .clone_from(&self.last_configuration);
        configure.protocol_version = self.protocol_version.clone();

        Ok(Message::new(configure))
    }
}

impl<'a> ProtocolsConfigureBuilder<'a> {
    pub fn new(actor: &'a Actor, definition: Option<ProtocolDefinition>) -> Self {
        ProtocolsConfigureBuilder {
            actor,
            authorized: true,
            definition,
            last_configuration: None,
            protocol_version: Version::new(0, 0, 0),
            target: None,
        }
    }

    /// Set the CID of the last configuration.
    /// Used to update a version of a protocol.
    pub fn last_configuration(mut self, configuration: String) -> Self {
        self.last_configuration = Some(configuration);
        self
    }

    /// Set the protocol version to be configured.
    /// Defaults to "0.0.0".
    pub fn protocol_version(mut self, version: Version) -> Self {
        self.protocol_version = version;
        self
    }

    pub async fn process(&mut self) -> Result<StatusReply, ProcessMessageError> {
        let reply = MessageBuilder::process(self).await?;

        let reply = match reply {
            MessageReply::Status(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(reply)
    }
}
