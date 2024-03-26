use std::sync::Arc;

use dwn::{
    actor::{Actor, CreateRecord},
    store::SurrealStore,
    DWN,
};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_sync() {
    // Start a DWN server.
    let dwn_osaka = {
        let store = SurrealStore::new().await.unwrap();
        DWN::from(store)
    };

    let actor_osaka = Actor::new_did_key(dwn_osaka.clone()).unwrap();

    tokio::spawn(async {
        let router = dwn_server::router(Arc::new(dwn_osaka));
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        axum::serve(listener, router).await.unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Create another DWN.
    let mut dwn_kyoto = {
        let store = SurrealStore::new().await.unwrap();
        DWN::from(store)
    };

    // Sync Kyoto with Osaka.
    let mut remote_sync = dwn_kyoto.sync_with("http://localhost:8080".to_string());

    let actor_kyoto = Actor::new_did_key(dwn_kyoto).unwrap();

    // Create a record in Kyoto.
    let data = "Hello, world!".bytes().collect::<Vec<_>>();

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
    assert_eq!(read.data, Some(data));
}
