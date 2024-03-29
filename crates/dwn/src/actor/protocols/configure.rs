use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    handlers::{MessageReply, StatusReply},
    message::{
        descriptor::{ProtocolDefinition, ProtocolsConfigure},
        Message,
    },
    store::{DataStore, MessageStore},
};

pub struct ProtocolsConfigureBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    authorized: bool,
    definition: Option<ProtocolDefinition>,
    last_configuration: Option<String>,
    protocol_version: Option<String>,
    target: Option<String>,
}

impl<'a, D: DataStore, M: MessageStore> MessageBuilder for ProtocolsConfigureBuilder<'a, D, M> {
    fn get_actor(&self) -> &Actor<impl DataStore, impl MessageStore> {
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
        configure.definition = self.definition.clone();
        configure.last_configuration = self.last_configuration.clone();
        configure.protocol_version = self.protocol_version.clone();

        Ok(Message::new(configure))
    }
}

impl<'a, D: DataStore, M: MessageStore> ProtocolsConfigureBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>, definition: Option<ProtocolDefinition>) -> Self {
        ProtocolsConfigureBuilder {
            actor,
            authorized: true,
            definition,
            last_configuration: None,
            protocol_version: None,
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
    pub fn protocol_version(mut self, version: String) -> Self {
        self.protocol_version = Some(version);
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