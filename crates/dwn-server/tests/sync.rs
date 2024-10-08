use dwn::{
    actor::Actor,
    message::{
        descriptor::{
            protocols::{ProtocolDefinition, ProtocolsFilter},
            records::Version,
            Descriptor,
        },
        Data,
    },
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tokio::net::TcpListener;
use tracing_test::traced_test;

struct TestContext {
    alice_kyoto: Actor,
    alice_osaka: Actor,
    osaka_url: String,
}

async fn setup_test() -> TestContext {
    let port = port_scanner::request_open_port().unwrap();

    // Start a DWN server.
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store_osaka = SurrealStore::new(db).await.unwrap();
    let dwn_osaka = DWN::from(store_osaka);
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
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store_kyoto = SurrealStore::new(db).await.unwrap();
    let dwn_kyoto = DWN::from(store_kyoto.clone());

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
#[traced_test]
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
        .data_format("application/json".to_string())
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
#[traced_test]
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
        .data_format("application/json".to_string())
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
#[traced_test]
async fn test_sync_pull_update() {
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
        .data_format("application/json".to_string())
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
        .data_format("application/json".to_string())
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

#[tokio::test]
#[traced_test]
async fn test_sync_pull_many_updates() {
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
        .data_format("application/json".to_string())
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
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Update the record in Osaka again.
    let newer_data = "Hello once more from Osaka!".bytes().collect::<Vec<_>>();
    let update = alice_osaka
        .update_record(create.record_id.clone(), update.entry_id.clone())
        .data(newer_data.clone())
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Kyoto should not have the updated records yet.
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
    assert_eq!(read.record.data, Some(Data::new_base64(&newer_data)));
}

#[tokio::test]
#[traced_test]
async fn test_sync_pull_delete() {
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
        .data_format("application/json".to_string())
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

    // Delete the record in Osaka.
    let delete = alice_osaka
        .delete_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(delete.reply.status.code, 200);

    // Kyoto should still have the record.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Sync data.
    alice_kyoto.sync().await.unwrap();

    // Kyoto should have deleted the record now.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, None);
    assert!(matches!(
        read.record.descriptor,
        Descriptor::RecordsDelete(_)
    ));
}

#[tokio::test]
#[traced_test]
async fn test_sync_pull_delete_after_update() {
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
        .data_format("application/json".to_string())
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
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    let read = alice_osaka
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&new_data)));

    // Delete the record in Osaka.
    let delete = alice_osaka
        .delete_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(delete.reply.status.code, 200);

    // Kyoto should still have the original record.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Sync data.
    alice_kyoto.sync().await.unwrap();

    // Kyoto should have deleted the record now.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, None);
    assert!(matches!(
        read.record.descriptor,
        Descriptor::RecordsDelete(_)
    ));
}

#[tokio::test]
#[traced_test]
async fn test_sync_pull_delete_after_local_update() {
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
        .data_format("application/json".to_string())
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

    // Update the record in Kyoto.
    let new_data = "Hello from Kyoto!".bytes().collect::<Vec<_>>();
    let update = alice_kyoto
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(new_data.clone())
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Delete the record in Osaka.
    let delete = alice_osaka
        .delete_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(delete.reply.status.code, 200);

    // Kyoto should still have the updated record.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&new_data)));

    // Sync.
    alice_kyoto.sync().await.unwrap();

    // Kyoto should have deleted the record now.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, None);
    assert!(matches!(
        read.record.descriptor,
        Descriptor::RecordsDelete(_)
    ));
}

#[tokio::test]
#[traced_test]
async fn test_sync_update_pulled() {
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
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Read the record in Kyoto.
    let read = alice_kyoto
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Update the record in Kyoto.
    let new_data = "Hello from Kyoto!".bytes().collect::<Vec<_>>();
    let update = alice_kyoto
        .update_record(create.record_id.clone(), create.entry_id.clone())
        .data(new_data.clone())
        .data_format("application/json".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(update.reply.status.code, 200);

    // Sync.
    alice_kyoto.sync().await.unwrap();

    // Read the update from Osaka.
    let read = alice_osaka
        .read_record(create.record_id.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&new_data)));
}

#[tokio::test]
#[traced_test]
async fn test_sync_protocols() {
    let TestContext {
        mut alice_kyoto,
        alice_osaka,
        osaka_url,
    } = setup_test().await;

    // Add the Osaka DWN as a remote.
    alice_kyoto.add_remote(osaka_url);

    // Register a protocol in Kyoto.
    let definition = ProtocolDefinition {
        published: true,
        protocol: "my-protocol-1".to_string(),
        ..Default::default()
    };

    let register = alice_kyoto
        .register_protocol(definition.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Sync.
    alice_kyoto.sync().await.unwrap();

    // Query the protocol in Osaka.
    let query = alice_osaka
        .query_protocols(ProtocolsFilter {
            protocol: definition.protocol,
            versions: vec![Version::new(0, 0, 0)],
        })
        .process()
        .await
        .unwrap();
    assert_eq!(query.status.code, 200);
    assert!(!query.entries.is_empty());
}
