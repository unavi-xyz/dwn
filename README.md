# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

### Usage

```rust
use dwn::{
    handlers::Status,
    message::{Data, MessageBuilder, descriptor::RecordsWrite},
    store::SurrealDB,
    util::DidKey,
    DWN
};

#[tokio::main]
async fn main() {
    // Create a DWN, using an embedded SurrealDB instance as both the data and message store.
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    // Generate a DID.
    let did_key = DidKey::new().unwrap();

    // Store a record in the DWN.
    let message = MessageBuilder::new::<RecordsWrite>()
        .authorize(did_key.kid.clone(), &did_key.jwk)
        .data(Data::Base64("Hello, world!".to_string()))
        .build()
        .unwrap();

    let reply = dwn
        .process_message(&did_key.did, message)
        .await
        .unwrap();

    assert_eq!(reply.status().code, 200);
}
```

<!-- cargo-rdme end -->
