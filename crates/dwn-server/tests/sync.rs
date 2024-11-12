use dwn::{
    builders::records::RecordsWriteBuilder,
    core::{message::mime::TEXT_PLAIN, store::RecordStore},
};
use tracing_test::traced_test;
use utils::init_test;

mod utils;

#[tokio::test]
#[traced_test]
async fn test_sync_write() {
    let (actor, mut dwn, remote) = init_test().await;

    let data = "Hello, world!".as_bytes().to_vec();
    let mut msg = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    actor.authorize(&mut msg).unwrap();
    let record_id = msg.record_id.clone();

    dwn.process_message(&actor.did, msg.clone()).await.unwrap();
    dwn.sync().await.unwrap();

    let found = remote.read(&actor.did, &record_id, true).unwrap().unwrap();
    assert_eq!(found, msg);
}

#[tokio::test]
#[traced_test]
async fn test_sync_update() {
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

    dwn.sync().await.unwrap();

    let found = remote.read(&actor.did, &record_id, true).unwrap().unwrap();
    assert_eq!(found, msg_2);
}
