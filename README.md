# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

### Usage

```rust
use std::sync::Arc;

use dwn::{actor::Actor, message::Data, store::SurrealStore, DWN};
use surrealdb::{Surreal, engine::local::Mem};

#[tokio::main]
async fn main() {
    // Create a DWN, using in-memory SurrealDB for storage.
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    // Create an actor to send messages.
    // Here we generate a new `did:key` for the actor's identity,
    // but you could use any DID method.
    let actor = Actor::new_did_key(dwn).unwrap();

    // Create a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    let create = actor
        .create_record()
        .data(data.clone())
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();

    assert_eq!(create.reply.status.code, 200);

    // Read the record.
    let read = actor
        .read_record(create.record_id)
        .process()
        .await
        .unwrap();

    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));
}
```

<!-- cargo-rdme end -->
