use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::Data,
    store::{DataStore, MessageStore, SurrealStore},
    DWN,
};
use tokio::net::TcpListener;

struct TestContext<D: DataStore, M: MessageStore> {
    alice_kyoto: Actor<D, M>,
    alice_osaka: Actor<D, M>,
    osaka_url: String,
}

async fn setup_test() -> TestContext<impl DataStore, impl MessageStore> {
    let port = port_scanner::request_open_port().unwrap();

    // Start a DWN server.
    let store_osaka = SurrealStore::new().await.unwrap();
    let dwn_osaka = Arc::new(DWN::from(store_osaka));
    let alice_osaka = Actor::new_did_key(dwn_osaka.clone()).unwrap();

    tokio::spawn(async move {
        let router = dwn_server::router(dwn_osaka);
        let url = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(url).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });

    // Wait for the server to start.
    while port_scanner::scan_port(port) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Create another DWN.
    let store_kyoto = SurrealStore::new().await.unwrap();
    let dwn_kyoto = Arc::new(DWN::from(store_kyoto.clone()));

    let alice_kyoto = Actor {
        attestation: alice_osaka.attestation.clone(),
        authorization: alice_osaka.authorization.clone(),
        did: alice_osaka.did.clone(),
        dwn: dwn_kyoto,
        remotes: Vec::new(),
    };

    let osaka_url = format!("http://localhost:{}", port);

    TestContext {
        alice_kyoto,
        alice_osaka,
        osaka_url,
    }
}

#[tokio::test]
async fn test_read_remote() {
    let TestContext {
        mut alice_kyoto,
        alice_osaka,
        osaka_url,
    } = setup_test().await;

    // Add the Osaka DWN as a remote.
    alice_kyoto.add_remote(osaka_url.clone());

    // Create a record in Osaka.
    let data = "Hello from Osaka!".bytes().collect::<Vec<_>>();
    let create = alice_osaka
        .create_record()
        .data(data.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Kyoto should be able to read the record.
    // Kyoto will fetch the remote if a record is not found.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // If we remove the remote, Kyoto should still be able to read the record.
    // The record is now stored locally.
    alice_kyoto.remove_remote(&osaka_url);

    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));
}

#[tokio::test]
async fn test_sync_push() {
    let TestContext {
        mut alice_kyoto,
        alice_osaka,
        osaka_url,
    } = setup_test().await;

    // Add the Osaka DWN as a remote.
    alice_kyoto.add_remote(osaka_url);

    // Create a record in Kyoto.
    let data = "Hello from Kyoto!".bytes().collect::<Vec<_>>();
    let create = alice_kyoto
        .create_record()
        .data(data.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Osaka should not have the record yet.
    let read = alice_osaka
        .read_record(create.record_id.clone())
        .process()
        .await;
    assert!(read.is_err());

    // Sync data.
    alice_kyoto.sync().await.unwrap();

    // Osaka should have the record now.
    let read = alice_osaka
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));
}

#[tokio::test]
async fn test_sync_pull() {
    let TestContext {
        mut alice_kyoto,
        alice_osaka,
        osaka_url,
    } = setup_test().await;

    // Add the Osaka DWN as a remote.
    alice_kyoto.add_remote(osaka_url);

    // Create a record in Osaka.
    let data = "Hello from Osaka!".bytes().collect::<Vec<_>>();
    let create = alice_osaka
        .create_record()
        .data(data.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Read the record in Kyoto.
    // This will store the record locally.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Update the record in Osaka.
    let new_data = "Hello again from Osaka!".bytes().collect::<Vec<_>>();
    let update = alice_osaka
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(new_data.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Kyoto should not have the updated record yet.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Sync data.
    alice_kyoto.sync().await.unwrap();

    // Kyoto should have the updated record now.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&new_data)));
}
