use dwn::builders::records::write::RecordsWriteBuilder;
use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

use super::expect_success;

#[tokio::test]
#[traced_test]
async fn test_update() {
    let (actor, dwn) = init_dwn();

    let data_1 = "hello, world!".as_bytes().to_owned();
    let mut msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data_1)
        .build()
        .unwrap();
    actor.authorize(&mut msg_1).unwrap();

    let record_id = msg_1.record_id.clone();
    expect_success(&actor.did, &dwn, msg_1).await;

    let data_2 = "goodbye".as_bytes().to_owned();
    let mut msg_2 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_2)
        .parent_id(record_id.clone())
        .build()
        .unwrap();
    actor.authorize(&mut msg_2).unwrap();

    let msg_2_id = msg_2.descriptor.compute_entry_id().unwrap();
    expect_success(&actor.did, &dwn, msg_2).await;

    let data_3 = "123".as_bytes().to_owned();
    let mut msg_3 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_3)
        .parent_id(msg_2_id)
        .build()
        .unwrap();
    actor.authorize(&mut msg_3).unwrap();

    expect_success(&actor.did, &dwn, msg_3).await;
}

#[tokio::test]
#[traced_test]
async fn test_update_require_parent() {
    let (actor, dwn) = init_dwn();

    let data_1 = "hello, world!".as_bytes().to_owned();
    let mut msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data_1)
        .build()
        .unwrap();
    actor.authorize(&mut msg_1).unwrap();

    let record_id = msg_1.record_id.clone();
    expect_success(&actor.did, &dwn, msg_1.clone()).await;

    let data_2 = "goodbye".as_bytes().to_owned();
    let mut msg_2 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_2)
        .build()
        .unwrap();
    actor.authorize(&mut msg_2).unwrap();

    assert!(dwn.process_message(&actor.did, msg_2).await.is_err());
    assert_eq!(
        dwn.record_store
            .read(&actor.did, &record_id, true)
            .unwrap()
            .unwrap(),
        msg_1
    );
}

#[tokio::test]
#[traced_test]
async fn test_update_wrong_parent() {
    let (actor, dwn) = init_dwn();

    let data_1 = "hello, world!".as_bytes().to_owned();
    let mut msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data_1)
        .build()
        .unwrap();
    actor.authorize(&mut msg_1).unwrap();
    let record_id = msg_1.record_id.clone();
    expect_success(&actor.did, &dwn, msg_1.clone()).await;

    let mut msg_2 = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg_2).unwrap();
    let parent_id = msg_2.record_id.clone();
    expect_success(&actor.did, &dwn, msg_2.clone()).await;

    let data_3 = "goodbye".as_bytes().to_owned();
    let mut msg_3 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_3)
        .parent_id(parent_id)
        .build()
        .unwrap();
    actor.authorize(&mut msg_3).unwrap();

    // Should not error, but msg_3 should be discarded and not written.
    dwn.process_message(&actor.did, msg_3).await.unwrap();

    assert_eq!(
        dwn.record_store
            .read(&actor.did, &record_id, true)
            .unwrap()
            .unwrap(),
        msg_1
    );
}
