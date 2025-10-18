use dwn_core::message::mime::TEXT_PLAIN;
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_actor_write_read() {
    let (actor, ..) = init_dwn();

    let data = "Hello, world!".as_bytes().to_owned();

    let record_id = actor
        .write()
        .data(TEXT_PLAIN, data.clone())
        .process()
        .await
        .expect("write");

    let found = actor
        .read(record_id.clone())
        .process()
        .await
        .expect("read")
        .expect("record is found");
    assert_eq!(found.entry().record_id, record_id);

    let found_data = found.data().unwrap();
    assert_eq!(found_data, data)
}

#[tokio::test]
#[traced_test]
async fn test_actor_query() {
    let (actor, ..) = init_dwn();

    let data_1 = "Hello, world!".as_bytes().to_owned();
    let data_2 = "Goodbye, world!".as_bytes().to_owned();

    let id_1 = actor
        .write()
        .data(TEXT_PLAIN, data_1.clone())
        .process()
        .await
        .expect("write");

    let id_2 = actor
        .write()
        .data(TEXT_PLAIN, data_2.clone())
        .process()
        .await
        .expect("write");

    let found = actor.query().process().await.unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|x| x.entry().record_id == id_1));
    assert!(found.iter().any(|x| x.entry().record_id == id_2));
}
