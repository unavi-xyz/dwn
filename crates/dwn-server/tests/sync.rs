use std::sync::Arc;

use dwn::{
    actor::{Actor, MessageBuilder},
    message::data::Data,
    store::SurrealStore,
    DWN,
};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_sync() {
    let port = port_scanner::request_open_port().unwrap();

    // Start a DWN server.
    let store_osaka = SurrealStore::new().await.unwrap();
    let dwn_osaka = Arc::new(DWN::from(store_osaka));
    let actor_osaka = Actor::new_did_key(dwn_osaka.clone()).unwrap();

    tokio::spawn(async move {
        let router = dwn_server::router(dwn_osaka);
        let url = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(url).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Create another DWN.
    let store_kyoto = SurrealStore::new().await.unwrap();
    let dwn_kyoto = Arc::new(DWN::from(store_kyoto.clone()));
    let mut actor_kyoto = Actor::new_did_key(dwn_kyoto.clone()).unwrap();

    // Add the Osaka DWN as a remote.
    let osaka_url = format!("http://localhost:{}", port);
    actor_kyoto.add_remote(osaka_url.clone());

    // Create a record in Kyoto.
    let data = "Hello from Kyoto!".bytes().collect::<Vec<_>>();
    let create = actor_kyoto
        .create()
        .data(data.clone())
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Osaka should not have the record yet.
    let read = actor_osaka
        .read(create.record_id.clone())
        .target(actor_kyoto.did.clone())
        .process()
        .await;
    assert!(read.is_err());

    // Sync data.
    actor_kyoto.sync().await.unwrap();

    // Osaka should have the record now.
    let read = actor_osaka
        .read(create.record_id.clone())
        .target(actor_kyoto.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Create a record in Osaka.
    let data = "Hello from Osaka!".bytes().collect::<Vec<_>>();
    let create = actor_osaka
        .create()
        .data(data.clone())
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Kyoto should be able to read the record.
    // Kyoto will fetch the remote if a record is not found.
    let read = actor_kyoto
        .read(create.record_id.clone())
        .target(actor_osaka.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // If we remove the remote, Kyoto should still be able to read the record.
    // The record is now stored locally.
    actor_kyoto.remove_remote(&osaka_url);

    let read = actor_kyoto
        .read(create.record_id.clone())
        .target(actor_osaka.did.clone())
        .process()
        .await
        .unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Add the remote back.
    actor_kyoto.add_remote(osaka_url);
}
