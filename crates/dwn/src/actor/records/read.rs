use crate::{
    actor::{Actor, MessageBuilder, PrepareError, ProcessMessageError},
    handlers::{RecordsReadReply, Reply},
    message::{descriptor::RecordsRead, Message, Request},
    store::{DataStore, MessageStore},
    HandleMessageError,
};

pub struct RecordsReadBuilder<'a, D: DataStore, M: MessageStore> {
    actor: &'a Actor<D, M>,
    authorized: bool,
    record_id: String,
    target: Option<String>,

    final_message: Option<Message>,
}

impl<'a, D: DataStore, M: MessageStore> MessageBuilder for RecordsReadBuilder<'a, D, M> {
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
        Ok(Message::new(RecordsRead::new(self.record_id.clone())))
    }

    fn post_build(&mut self, message: &mut Message) -> Result<(), PrepareError> {
        self.final_message = Some(message.clone());
        Ok(())
    }
}

impl<'a, D: DataStore, M: MessageStore> RecordsReadBuilder<'a, D, M> {
    pub fn new(actor: &'a Actor<D, M>, record_id: String) -> Self {
        RecordsReadBuilder {
            actor,
            authorized: true,
            record_id,
            target: None,

            final_message: None,
        }
    }

    pub async fn process(&mut self) -> Result<RecordsReadReply, ProcessMessageError> {
        let reply = match MessageBuilder::process(self).await {
            Ok(Reply::RecordsRead(reply)) => reply,
            Ok(_) => unreachable!(),
            Err(err) => {
                let message = self.final_message.as_ref().unwrap();

                // Check remotes for record.
                for remote in &self.actor.remotes {
                    let target = self
                        .target
                        .clone()
                        .unwrap_or_else(|| self.actor.did.clone());

                    let request = Request {
                        target: target.clone(),
                        message: message.clone(),
                    };

                    let reply = self
                        .actor
                        .client
                        .post(remote.url())
                        .json(&request)
                        .send()
                        .await?
                        .json::<Reply>()
                        .await;

                    if let Ok(Reply::RecordsRead(reply)) = reply {
                        // Store the record locally.
                        self.actor
                            .dwn
                            .message_store
                            .put(target, *reply.record.clone(), &self.actor.dwn.data_store)
                            .await
                            .map_err(HandleMessageError::MessageStoreError)?;

                        return Ok(reply);
                    }
                }

                return Err(err);
            }
        };

        Ok(reply)
    }
}
