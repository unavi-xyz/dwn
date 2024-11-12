# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

The DWN spec is a work-in-progress and often out of date from other implementations,
so it is treated more as a loose guide rather than an absolute set of rules to follow.

## Example

```rust
use dwn::{
    builders::records::{RecordsReadBuilder, RecordsWriteBuilder},
    core::{message::mime::TEXT_PLAIN, reply::Reply},
    stores::NativeDbStore,
    Actor,
    Dwn
};
use xdid::methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey};

#[tokio::main]
async fn main() {
    // Create a local in-memory DWN.
    let store = NativeDbStore::new_in_memory().unwrap();
    let mut dwn = Dwn::from(store);
   
    // Create a new did:key.
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
   
    // Create an actor to sign messages on behalf of the DID.
    let mut actor = Actor::new(did.clone());
    actor.auth_key = Some(key.clone().into());
    actor.sign_key = Some(key.into());
   
    // Prepare to write a new record to the DWN.
    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_owned())
        .published(true)
        .build()
        .unwrap();
   
    let record_id = msg.record_id.clone();
   
    // Authorize the message using our actor.
    actor.authorize(&mut msg).unwrap();
   
    // Process the write.
    dwn.process_message(&did, msg).await.unwrap();
   
    // We can now read the record using its ID.
    let msg = RecordsReadBuilder::new(record_id.clone())
        .build()
        .unwrap();
   
    let reply = dwn.process_message(&did, msg).await.unwrap();

    let record = match reply {
        Some(Reply::RecordsRead(r)) => r.entry.unwrap(),
        _ => panic!("invalid reply"),
    };

    assert_eq!(record.record_id, record_id);
}
```

<!-- cargo-rdme end -->
