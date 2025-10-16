# dwn

<!-- cargo-rdme start -->

Rust implementation of a [Decentralized Web Node](https://identity.foundation/decentralized-web-node/spec/).

The DWN spec is a work-in-progress and often out of date from other implementations,
so it is treated more as a loose guide rather than an absolute set of rules to follow.

## Example

```rust
use dwn::{
    core::{message::{descriptor::{RecordsReadBuilder, RecordsWriteBuilder}, mime::TEXT_PLAIN}, reply::Reply},
    stores::NativeDbStore,
    Actor,
    Dwn
};
use xdid::methods::key::{p256::P256KeyPair, DidKeyPair, PublicKey};

#[tokio::main]
async fn main() {
    // Create a local in-memory DWN.
    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);
   
    // Create a new did:key.
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
   
    // Create an actor to sign messages on behalf of our DID.
    let mut actor = Actor::new(did, dwn);
    actor.auth_key = Some(key.clone().into());
    actor.sign_key = Some(key.into());
   
    // Write a new record to the DWN.
    let data = "Hello, world!".as_bytes().to_vec();

    let record_id = actor.write()
        .data(TEXT_PLAIN, data.clone())
        .published(true)
        .process()
        .await
        .unwrap();

    // We can now read the record using its ID.
    let found = actor.read(record_id.clone())
        .process()
        .await
        .unwrap()
        .unwrap();

   assert!(found.entry().record_id, record_id);
   assert!(found.data().unwrap(), data);
}
```

<!-- cargo-rdme end -->
