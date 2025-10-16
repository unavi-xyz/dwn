use dwn::core::{
    message::{descriptor::RecordsWriteBuilder, mime::TEXT_PLAIN},
    store::RecordStore,
};
use tracing_test::traced_test;
use utils::init_remote_test;

mod utils;

#[tokio::test]
#[traced_test]
async fn test_auto_sync_local_write() {
    let (actor, dwn, remote) = init_remote_test().await;

    let record_id = actor
        .write()
        .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_vec())
        .process()
        .await
        .unwrap();

    let found = remote
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry.record_id, record_id);
}

#[tokio::test]
#[traced_test]
async fn test_manual_sync_local_write() {
    let (actor, dwn, remote) = init_remote_test().await;

    let record_id = actor
        .write()
        .data(TEXT_PLAIN, "Hello, world!".as_bytes().to_vec())
        .sync(false)
        .process()
        .await
        .unwrap();
    assert!(
        remote
            .read(dwn.data_store.as_ref(), &actor.did, &record_id)
            .unwrap()
            .is_none()
    );

    actor.sync().await.unwrap();

    let found = remote
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry.record_id, record_id);
}

#[tokio::test]
#[traced_test]
async fn test_auto_sync_local_update() {
    let (actor, dwn, remote) = init_remote_test().await;

    let data = "Hello, world!".as_bytes().to_vec();
    let record_id = actor
        .write()
        .data(TEXT_PLAIN, data)
        .process()
        .await
        .unwrap();

    let data_2 = "Goodbye, world!".as_bytes().to_vec();
    actor
        .write()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_2)
        .process()
        .await
        .unwrap();

    let found = remote
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry.record_id, record_id);
}

#[tokio::test]
#[traced_test]
async fn test_sync_remote_write() {
    let (actor, dwn, remote) = init_remote_test().await;

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

    actor.sync().await.unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.latest_entry, msg);
}
