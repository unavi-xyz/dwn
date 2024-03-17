# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

### Usage

```rust
use std::sync::Arc;

use dwn::{
    handlers::Status,
    message::Data,
    store::SurrealDB,
    Actor,
    DWN
};

#[tokio::main]
async fn main() {
    // Create a DWN, using embedded SurrealDB for both the data and message store.
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN {
        data_store: db.clone(),
        message_store: db,
    };

    // Create an actor to send messages.
    // Here we generate a new `did:key` for the actor's identity,
    // but you could use any DID method.
    let actor = Actor::new_did_key(dwn).unwrap();

    // Write a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

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
    assert_eq!(reply.data, Some(data));
}
```

<!-- cargo-rdme end -->
