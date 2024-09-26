use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    message::{
        descriptor::{records::RecordsRead, Descriptor},
        DwnRequest, Message,
    },
    reply::{MessageReply, RecordsReadReply},
    HandleMessageError,
};

pub struct RecordsReadBuilder<'a> {
    actor: &'a Actor,
    authorized: bool,
    record_id: String,
    target: Option<String>,

    final_message: Option<Message>,
}

impl<'a> MessageBuilder for RecordsReadBuilder<'a> {
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
        Ok(Message::new(RecordsRead::new(self.record_id.clone())))
    }

    fn post_build(&mut self, message: &mut Message) -> Result<(), PrepareError> {
        self.final_message = Some(message.clone());
        Ok(())
    }
}

impl<'a> RecordsReadBuilder<'a> {
    pub fn new(actor: &'a Actor, record_id: String) -> Self {
        RecordsReadBuilder {
            actor,
            authorized: true,
            record_id,
            target: None,

            final_message: None,
        }
    }

    pub async fn process(&mut self) -> Result<RecordsReadReply, ProcessMessageError> {
        match MessageBuilder::process(self).await {
            Ok(MessageReply::RecordsRead(reply)) => {
                let data_cid = match &reply.record.descriptor {
                    Descriptor::RecordsWrite(descriptor) => descriptor.data_cid.clone(),
                    _ => None,
                };

                // If we don't have the data, check remote.
                if data_cid.is_some() && reply.record.data.is_none() {
                    if let Some(found) = self.read_remote().await? {
                        return Ok(found);
                    }
                }

                Ok(reply)
            }
            Ok(_) => unreachable!(),
            Err(err) => {
                // Check remote.
                if let Some(found) = self.read_remote().await? {
                    // Store the record locally.
                    // TODO: Only store data locally if under some size
                    let target = self
                        .target
                        .clone()
                        .unwrap_or_else(|| self.actor.did.clone());

                    self.actor
                        .dwn
                        .message_store
                        .put(
                            true,
                            target,
                            *found.record.clone(),
                            self.actor.dwn.data_store.clone(),
                        )
                        .await
                        .map_err(HandleMessageError::MessageStoreError)?;

                    return Ok(found);
                }

                Err(err)
            }
        }
    }

    pub async fn send(mut self, did: &str) -> Result<RecordsReadReply, ProcessMessageError> {
        let reply = MessageBuilder::send(&mut self, did).await?;

        let reply = match reply {
            MessageReply::RecordsRead(reply) => reply,
            _ => return Err(ProcessMessageError::InvalidReply),
        };

        Ok(reply)
    }

    async fn read_remote(&self) -> Result<Option<RecordsReadReply>, ProcessMessageError> {
        let target = self
            .target
            .clone()
            .unwrap_or_else(|| self.actor.did.clone());

        let message = self.final_message.as_ref().unwrap();

        for remote in &self.actor.remotes {
            let request = DwnRequest {
                target: target.clone(),
                message: message.clone(),
            };

            let reply = self
                .actor
                .dwn
                .client
                .post(remote.url())
                .json(&request)
                .send()
                .await?
                .json::<MessageReply>()
                .await?;

            if let MessageReply::RecordsRead(reply) = reply {
                return Ok(Some(reply));
            }
        }

        Ok(None)
    }
}
