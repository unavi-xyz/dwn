use dwn::builders::records::write::RecordsWriteBuilder;
use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

use super::expect_success;

#[tokio::test]
#[traced_test]
async fn test_update() {
    let (actor, mut dwn) = init_dwn();

    let data_1 = "hello, world!".as_bytes().to_owned();
    let mut msg_1 = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data_1)
        .build()
        .unwrap();
    actor.authorize(&mut msg_1).unwrap();

    let record_id = msg_1.record_id.clone();
    expect_success(&actor.did, &mut dwn, msg_1).await;

    let data_2 = "goodbye".as_bytes().to_owned();
    let mut msg_2 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_2)
        .build()
        .unwrap();
    actor.authorize(&mut msg_2).unwrap();

    expect_success(&actor.did, &mut dwn, msg_2).await;

    let data_3 = "123".as_bytes().to_owned();
    let mut msg_3 = RecordsWriteBuilder::default()
        .record_id(record_id.clone())
        .data(TEXT_PLAIN, data_3)
        .build()
        .unwrap();
    actor.authorize(&mut msg_3).unwrap();

    expect_success(&actor.did, &mut dwn, msg_3).await;
}
