# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

### Usage

```rust
use dwn::{actor::{Actor, CreateRecord}, store::SurrealDB, DWN};

#[tokio::main]
async fn main() {
    // Create a DWN, using an embedded SurrealDB for storage.
    let db = SurrealDB::new().await.unwrap();
    let dwn = DWN::new(db);

    // Create an actor to send messages.
    // Here we generate a new `did:key` for the actor's identity,
    // but you could use any DID method.
    let actor = Actor::new_did_key(dwn).unwrap();

    // Create a new record.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

    let create = actor
        .create(CreateRecord {
            data: Some(data.clone()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(create.reply.status.code, 200);

    // Read the record.
    let read = actor.read(create.record_id).await.unwrap();

    assert_eq!(read.status.code, 200);
    assert_eq!(read.data, Some(data));
}
```

<!-- cargo-rdme end -->
