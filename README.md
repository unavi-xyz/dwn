# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

### Usage

```rust
use std::sync::Arc;

use dwn::{
    actor::Actor,
    handlers::Status,
    message::Data,
    store::SurrealDB,
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

    // Create an actor to send messages.
    // Here we generate a new `did:key` for the actor's identity, but this could be any DID method.
    let actor = Actor::new_did_key(dwn).unwrap();

    // Write a new record.
    let data = Data::Base64("Hello, world!".to_string());

    let res = actor
        .write()
        .data(data.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(res.reply.status.code, 200);

    // Read the record.
    let reply = actor.read(res.record_id).await.unwrap();

    assert_eq!(reply.status.code, 200);
    assert_eq!(reply.data, Some(data.into()));
}
```

<!-- cargo-rdme end -->
