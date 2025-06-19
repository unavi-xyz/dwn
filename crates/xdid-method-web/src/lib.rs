//! [xdid](https://github.com/unavi-xyz/xdid) implementation of [did:web](https://w3c-ccg.github.io/did-method-web/).

use reqwest::{Client, ClientBuilder};
use xdid_core::{Method, ResolutionError, did::Did, document::Document};

pub use reqwest;

mod parse;

const NAME: &str = "web";

pub struct MethodDidWeb {
    pub client: Client,
}

impl MethodDidWeb {
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = ClientBuilder::new().build()?;
        Ok(Self { client })
    }
}

impl Method for MethodDidWeb {
    fn method_name(&self) -> &'static str {
        NAME
    }

    fn resolve(
        &self,
        did: Did,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<xdid_core::document::Document, ResolutionError>>
                + Send
                + Sync,
        >,
    > {
        debug_assert_eq!(did.method_name.0, self.method_name());

        let client = self.client.clone();
        let url = parse::parse_url(&did);

        Box::pin(async move {
            let req = client
                .get(url)
                .build()
                .map_err(|_| ResolutionError::InvalidDid)?;

            let doc = client
                .execute(req)
                .await
                .map_err(|e| ResolutionError::ResolutionFailed(e.to_string()))?
                .json::<Document>()
                .await
                .map_err(|e| ResolutionError::ResolutionFailed(e.to_string()))?;

            Ok(doc)
        })
    }
}
