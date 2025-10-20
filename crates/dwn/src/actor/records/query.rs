use anyhow::bail;
use dwn_core::{
    message::{
        Message, Version,
        descriptor::{DateFilter, DateSort, RecordsQueryBuilder},
        mime::Mime,
    },
    reply::Reply,
};
use reqwest::Url;
use xdid::core::did::Did;

use crate::{Actor, records::RecordView};

impl Actor {
    pub fn query(&self) -> ActorQueryBuilder<'_> {
        ActorQueryBuilder {
            actor: self,
            msg: RecordsQueryBuilder::default(),
            auth: true,
            target: None,
        }
    }
}

pub struct ActorQueryBuilder<'a> {
    actor: &'a Actor,
    msg: RecordsQueryBuilder,
    auth: bool,
    target: Option<&'a Did>,
}

impl<'a> ActorQueryBuilder<'a> {
    pub fn attester(mut self, value: Did) -> Self {
        self.msg.filter.attester = Some(value);
        self
    }

    pub fn recipient(mut self, value: Did) -> Self {
        self.msg.filter.recipient = Some(value);
        self
    }

    pub fn schema(mut self, value: String) -> Self {
        self.msg.filter.schema = Some(value);
        self
    }

    pub fn record_id(mut self, value: String) -> Self {
        self.msg.filter.record_id = Some(value);
        self
    }

    pub fn protocol(mut self, value: String) -> Self {
        self.msg.filter.protocol = Some(value);
        self
    }

    pub fn protocol_path(mut self, value: String) -> Self {
        self.msg.filter.protocol_path = Some(value);
        self
    }

    pub fn protocol_version(mut self, value: Version) -> Self {
        self.msg.filter.protocol_version = Some(value);
        self
    }

    pub fn data_format(mut self, value: Mime) -> Self {
        self.msg.filter.data_format = Some(value);
        self
    }

    pub fn date_created(mut self, value: DateFilter) -> Self {
        self.msg.filter.date_created = Some(value);
        self
    }

    pub fn date_sort(mut self, value: DateSort) -> Self {
        self.msg.filter.date_sort = Some(value);
        self
    }

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

    /// Sends the message to a remote DWN.
    pub async fn send(self, url: &Url) -> anyhow::Result<String> {
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;
        let id = msg.record_id.clone();

        actor.send(target, &msg, url).await?;

        Ok(id)
    }

    /// Processes the message with the actor's DWN.
    pub async fn process(self) -> anyhow::Result<Vec<RecordView>> {
        let actor = self.actor;
        let target = self.target.unwrap_or(&actor.did);

        let msg = self.build()?;

        let reply = actor
            .dwn
            .process_message(target, msg)
            .await
            .map_err(|e| anyhow::anyhow!("failed to process message: {e}"))?;

        match reply {
            Some(Reply::RecordsQuery(query)) => Ok(query
                .entries
                .into_iter()
                .map(RecordView::from_entry)
                .collect::<Result<Vec<_>, _>>()?),
            Some(other) => {
                bail!("got invalid reply from DWN: {other:?}")
            }
            None => {
                bail!("got no reply from DWN")
            }
        }
    }
}
