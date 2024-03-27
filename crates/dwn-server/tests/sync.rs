use std::sync::Arc;

use dwn::{
    actor::{Actor, CreateRecord},
    message::data::Data,
    store::SurrealStore,
    Remote, DWN,
};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_sync() {
    let port = port_scanner::request_open_port().unwrap();

    // Start a DWN server.
    let dwn_osaka = {
        let store = SurrealStore::new().await.unwrap();
        Arc::new(DWN::from(store))
    };

    let actor_osaka = Actor::new_did_key(dwn_osaka.clone()).unwrap();

    tokio::spawn(async move {
        let router = dwn_server::router(dwn_osaka);
        let url = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(url).await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Create another DWN.
    let dwn_kyoto = {
        let store = SurrealStore::new().await.unwrap();
        Arc::new(DWN::from(store))
    };

    // Sync Kyoto with Osaka.
    let mut remote_sync = dwn_kyoto
        .set_remote(format!("http://localhost:{}", port))
        .await;

    let actor_kyoto = Actor::new_did_key(dwn_kyoto.clone()).unwrap();

    // Create a record in Kyoto.
    let data = "Hello from Kyoto!".bytes().collect::<Vec<_>>();
    let create = actor_kyoto
        .create(CreateRecord {
            data: Some(data.clone()),
            published: true,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Osaka should not have the record yet.
    let read = actor_osaka.read(create.record_id.clone()).await;
    assert!(read.is_err());

    // Sync data.
    let res = remote_sync.sync().await.unwrap().unwrap();
    assert_eq!(res.status.unwrap().code, 200);
    assert_eq!(res.replies[0].status().code, 200);

    // Osaka should have the record now.
    let read = actor_osaka.read(create.record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Create a record in Osaka.
    let data = "Hello from Osaka!".bytes().collect::<Vec<_>>();
    let create = actor_osaka
        .create(CreateRecord {
            data: Some(data.clone()),
            published: true,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(create.reply.status.code, 200);

    // Kyoto should be able to read the record.
    let read = actor_kyoto.read(create.record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // If we remove the remote, Kyoto should still be able to read the record.
    // This is because the record is now stored locally.
    dwn_kyoto.remote.write().await.take();

    let read = actor_kyoto.read(create.record_id.clone()).await.unwrap();
    assert_eq!(read.status.code, 200);
    assert_eq!(read.record.data, Some(Data::new_base64(&data)));

    // Add the remote back.
    *dwn_kyoto.remote.write().await = Some(Remote::from_remote_sync(&remote_sync));
}
