use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_delete() {
    let (actor, dwn) = init_dwn();

    let data = "Hello, world!".as_bytes().to_owned();

    let record_id = actor
        .write()
        .data(TEXT_PLAIN, data)
        .process()
        .await
        .unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.initial_entry.record_id, record_id);
    assert_eq!(found.latest_entry.record_id, record_id);

    actor.delete(record_id.clone()).process().await.unwrap();

    assert!(
        dwn.record_store
            .read(dwn.data_store.as_ref(), &actor.did, &record_id)
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
#[traced_test]
async fn test_delete_requires_auth() {
    let (actor, dwn) = init_dwn();

    let data = "Hello, world!".as_bytes().to_owned();

    let record_id = actor
        .write()
        .data(TEXT_PLAIN, data)
        .process()
        .await
        .unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.initial_entry.record_id, record_id);
    assert_eq!(found.latest_entry.record_id, record_id);

    let res = actor.delete(record_id.clone()).auth(false).process().await;
    assert!(res.is_err());

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &actor.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.initial_entry.record_id, record_id);
    assert_eq!(found.latest_entry.record_id, record_id);
}
