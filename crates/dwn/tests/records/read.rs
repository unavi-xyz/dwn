use dwn::{
    builders::records::{read::RecordsReadBuilder, write::RecordsWriteBuilder},
    Reply,
};
use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_read_data() {
    let (actor, dwn) = init_dwn();

    let data = "hello, world!".as_bytes().to_owned();

    let msg_write = RecordsWriteBuilder::default()
        .data(TEXT_PLAIN, data)
        .build()
        .unwrap();
    dwn.record_store
        .write(&actor.did, msg_write.clone())
        .unwrap();

    let msg_read = RecordsReadBuilder::new(msg_write.record_id.clone())
        .build()
        .unwrap();
    let reply = match dwn.process_message(&actor.did, msg_read).await.unwrap() {
        Reply::RecordsRead(m) => m,
        _ => panic!("invalid reply"),
    };
    assert_eq!(*reply, msg_write);
}
