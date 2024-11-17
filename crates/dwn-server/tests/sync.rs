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

    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_vec())
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    dwn.process_message(&actor.did, msg.clone()).await.unwrap();
    dwn.sync(&actor.did, Some(&actor)).await.unwrap();

    let found = remote.read(&actor.did, &record_id).unwrap().unwrap();
    assert_eq!(found.latest_entry, msg);
}

#[tokio::test]
#[traced_test]
async fn test_sync_local_update() {
    let (actor, mut dwn, remote) = init_test().await;

    let data = "Hello, world!".as_bytes().to_vec();
    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    dwn.process_message(&actor.did, msg).await.unwrap();

    let data_2 = "Goodbye, world!".as_bytes().to_vec();
    let mut msg_2 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_2)
        .build()
        .unwrap();
    actor.authorize(&mut msg_2).unwrap();

    dwn.process_message(&actor.did, msg_2.clone())
        .await
        .unwrap();

    dwn.sync(&actor.did, Some(&actor)).await.unwrap();

    let found = remote.read(&actor.did, &record_id).unwrap().unwrap();
    assert_eq!(found.latest_entry, msg_2);
}

#[tokio::test]
#[traced_test]
async fn test_sync_remote_write() {
    let (actor, mut dwn, remote) = init_test().await;

    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_vec())
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    remote.write(&actor.did, msg.clone()).unwrap();

    dwn.sync(&actor.did, Some(&actor)).await.unwrap();

    let found = dwn
        .record_store
        .read(&actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry, msg);
}
