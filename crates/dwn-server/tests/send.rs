use std::sync::Arc;

use axum::{routing::get, Json, Router};
use didkit::{
    ssi::{
        did::{
            RelativeDIDURL, Service, ServiceEndpoint, VerificationMethod, VerificationMethodMap,
        },
        vc::OneOrMany,
    },
    Document,
};
use dwn::{
    actor::{Actor, MessageBuilder},
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tokio::net::TcpListener;
use tracing_test::traced_test;

const KEY_FRAGMENT: &str = "key-0";

#[tokio::test]
#[traced_test]
async fn test_send() {
    let port = port_scanner::request_open_port().unwrap();

    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store_osaka = SurrealStore::new(db).await.unwrap();
    let dwn_osaka = Arc::new(DWN::from(store_osaka));

    // Make alice a did:web.
    let alice_did = format!("did:web:localhost%3A{}", port);
    let mut alice_osaka = Actor::new_did_key(dwn_osaka.clone()).unwrap();
    let key_id = format!("{}#{}", alice_did, KEY_FRAGMENT);
    alice_osaka.attestation.key_id = key_id.clone();
    alice_osaka.authorization.key_id = key_id;
    alice_osaka.did = alice_did.clone();

    // Host Osaka on a server.
    {
        let alice_did = alice_did.clone();
        let jwk = alice_osaka.authorization.jwk.clone();

        tokio::spawn(async move {
            // Host alice's did:web document at the server.
            let mut document = Document::new(&alice_did);

            document.service = Some(vec![Service {
                id: format!("{}#dwn", alice_did),
                type_: OneOrMany::One("DWN".to_string()),
                property_set: None,
                service_endpoint: Some(OneOrMany::One(ServiceEndpoint::URI(format!(
                    "http://localhost:{}",
                    port
                )))),
            }]);

            document.verification_method =
                Some(vec![VerificationMethod::Map(VerificationMethodMap {
                    controller: alice_did.clone(),
                    id: format!("{}#{}", &alice_did, KEY_FRAGMENT),
                    public_key_jwk: Some(jwk.to_public()),
                    type_: "JsonWebKey2020".to_string(),
                    ..Default::default()
                })]);

            document.assertion_method =
                Some(vec![VerificationMethod::RelativeDIDURL(RelativeDIDURL {
                    fragment: Some(KEY_FRAGMENT.to_string()),
                    ..Default::default()
                })]);

            document.authentication =
                Some(vec![VerificationMethod::RelativeDIDURL(RelativeDIDURL {
                    fragment: Some(KEY_FRAGMENT.to_string()),
                    ..Default::default()
                })]);

            let document = Arc::new(document);
            let router = Router::new().route(
                "/.well-known/did.json",
                get(|| async move { Json(document.clone()) }),
            );

            let router = router.merge(dwn_server::router(dwn_osaka));

            let url = format!("0.0.0.0:{}", port);
            let listener = TcpListener::bind(url).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    }

    // Wait for the server to start.
    while port_scanner::scan_port(port) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Create a record in Osaka.
    let data = "Hello from Osaka!!".as_bytes();

    let create = alice_osaka
        .create_record()
        .data(data.to_vec())
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Create another DWN.
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store_kyoto = SurrealStore::new(db).await.unwrap();
    let dwn_kyoto = Arc::new(DWN::from(store_kyoto.clone()));

    let bob_kyoto = Actor::new_did_key(dwn_kyoto).unwrap();

    // Send a message to alice via their DID.
    let read = bob_kyoto
        .read_record(create.record_id)
        .target(alice_did.clone())
        .send(&alice_did)
        .await
        .unwrap();
    assert_eq!(read.status().code, 200);
}
