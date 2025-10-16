use dwn::core::{
    message::{descriptor::RecordsWriteBuilder, mime::TEXT_PLAIN},
    store::RecordStore,
};
use tracing_test::traced_test;
use utils::init_test;

mod utils;

#[tokio::test]
#[traced_test]
async fn test_sync_local_write() {
    let (actor, mut dwn, remote) = init_test().await;

    let mut msg = RecordsWriteBuilder {
        data_format: Some(TEXT_PLAIN),
        data: Some("Hello, world!".as_bytes().to_vec()),
        ..Default::default()
    }
    .build()
    .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    dwn.process_message(&actor.did, msg.clone()).await.unwrap();
    dwn.sync(&actor.did, Some(&actor)).await.unwrap();

    let found = remote
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry, msg);
}

#[tokio::test]
#[traced_test]
async fn test_sync_local_update() {
    let (actor, mut dwn, remote) = init_test().await;

    let data = "Hello, world!".as_bytes().to_vec();
    let mut msg = RecordsWriteBuilder {
        data_format: Some(TEXT_PLAIN),
        data: Some(data),
        ..Default::default()
    }
    .build()
    .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    dwn.process_message(&actor.did, msg).await.unwrap();

    let data_2 = "Goodbye, world!".as_bytes().to_vec();
    let mut msg_2 = RecordsWriteBuilder {
        record_id: Some(record_id.clone()),
        data_format: Some(TEXT_PLAIN),
        data: Some(data_2),
        ..Default::default()
    }
    .build()
    .unwrap();
    actor.authorize(&mut msg_2).unwrap();

    dwn.process_message(&actor.did, msg_2.clone())
        .await
        .unwrap();

    dwn.sync(&actor.did, Some(&actor)).await.unwrap();

    let found = remote
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry, msg_2);
}

#[tokio::test]
#[traced_test]
async fn test_sync_remote_write() {
    let (actor, mut dwn, remote) = init_test().await;

    let mut msg = RecordsWriteBuilder {
        data_format: Some(TEXT_PLAIN),
        data: Some("Hello, world!".as_bytes().to_vec()),
        ..Default::default()
    }
    .build()
    .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    remote
        .write(dwn.data_store.as_ref(), &actor.did, msg.clone())
        .unwrap();

    dwn.sync(&actor.did, Some(&actor)).await.unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry, msg);
}
