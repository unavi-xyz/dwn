use anyhow::{Context, bail};
use dwn_core::{
    message::{Message, descriptor::Descriptor},
    reply::Reply,
};
use reqwest::Url;
use tracing::warn;
use xdid::core::did::Did;

use crate::Actor;

impl Actor {
    pub(crate) async fn send_remote(&self, msg: &Message) -> anyhow::Result<Option<Reply>> {
        let Some(url) = &self.remote else {
            bail!("remote url not set")
        };
        let reply = send_message(&self.client, &self.did, msg, url).await?;
        Ok(reply)
    }

    /// Full sync with the remote DWN.
    /// If an actor is provided, it will be used to authorize the sync.
    pub async fn sync(&self) -> anyhow::Result<()> {
        let descriptor = Descriptor::RecordsSync(Box::new(
            self.dwn.record_store.prepare_sync(&self.did, true)?,
        ));

        let mut msg = Message {
            record_id: descriptor.compute_entry_id()?,
            context_id: None,
            data: None,
            descriptor,
            attestation: None,
            authorization: None,
        };

        self.authorize(&mut msg)?;

        let reply = match self.send_remote(&msg).await? {
            Some(Reply::RecordsSync(reply)) => reply,
            other => {
                bail!("invalid reply: {other:?}");
            }
        };

        // Process new records.
        for record in reply.remote_only {
            if let Err(e) = self
                .dwn
                .process_message(&self.did, record.initial_entry)
                .await
            {
                warn!("Failed to process message during DWN sync: {e:?}");
                continue;
            };

            if let Err(e) = self
                .dwn
                .process_message(&self.did, record.latest_entry)
                .await
            {
                warn!("Failed to process message during DWN sync: {e:?}");
            };
        }

        // Process conflicting entries.
        for entry in reply.conflict {
            if let Err(e) = self.dwn.process_message(&self.did, entry).await {
                warn!("Failed to process message during DWN sync: {e:?}");
            };
        }

        // Send local records to remote.
        for record_id in reply.local_only {
            let Some(record) =
                self.dwn
                    .record_store
                    .read(self.dwn.data_store.as_ref(), &self.did, &record_id)?
            else {
                continue;
            };

            self.send_remote(&record.initial_entry).await?;

            if record.latest_entry.descriptor.compute_entry_id()? != record.initial_entry.record_id
            {
                self.send_remote(&record.latest_entry).await?;
            }
        }

        Ok(())
    }
}

async fn send_message(
    client: &reqwest::Client,
    target: &Did,
    msg: &Message,
    url: &Url,
) -> anyhow::Result<Option<Reply>> {
    let mut url = url.clone();

    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid url"))?;
        path.push(&target.to_string());
    }

    let req = client.put(url).json(msg).build().context("build request")?;
    let res = client.execute(req).await.context("execute request")?;
    let reply = res.json::<Option<Reply>>().await.context("parse reply")?;

    Ok(reply)
}
