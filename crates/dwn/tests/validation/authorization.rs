use dwn_core::message::descriptor::RecordsWriteBuilder;
use tracing_test::traced_test;
use xdid::methods::key::{DidKeyPair, p256::P256KeyPair};

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_valid_authorization() {
    let (actor, mut dwn) = init_dwn();

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    assert!(dwn.process_message(&actor.did, msg).await.is_ok());
}

#[tokio::test]
#[traced_test]
async fn test_invalid_payload() {
    let (actor, mut dwn) = init_dwn();

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    msg.authorization.as_mut().unwrap().payload = "abcdefghijklmnop".to_string();

    assert!(dwn.process_message(&actor.did, msg).await.is_err());
}

#[tokio::test]
#[traced_test]
async fn test_empty_signatures() {
    let (actor, mut dwn) = init_dwn();

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    msg.authorization.as_mut().unwrap().signatures.clear();

    assert!(dwn.process_message(&actor.did, msg).await.is_err());
}

#[tokio::test]
#[traced_test]
async fn test_invalid_signature() {
    let (actor, mut dwn) = init_dwn();

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    msg.authorization.as_mut().unwrap().signatures[0].signature = "abcdefghijklmnop".to_string();

    assert!(dwn.process_message(&actor.did, msg).await.is_err());
}

#[tokio::test]
#[traced_test]
async fn test_multiple_signatures() {
    let (actor, mut dwn) = init_dwn();

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    let sig = msg.authorization.as_mut().unwrap().signatures[0].clone();
    msg.authorization.as_mut().unwrap().signatures.push(sig);

    assert!(dwn.process_message(&actor.did, msg).await.is_ok());
}

#[tokio::test]
#[traced_test]
async fn test_multiple_signatures_invalid() {
    let (actor, mut dwn) = init_dwn();

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    let mut sig = msg.authorization.as_mut().unwrap().signatures[0].clone();
    sig.signature = "abcdefghijklmnop".to_string();
    msg.authorization.as_mut().unwrap().signatures.push(sig);

    assert!(dwn.process_message(&actor.did, msg).await.is_err());
}

#[tokio::test]
#[traced_test]
async fn test_wrong_auth_key() {
    let (mut actor, mut dwn) = init_dwn();

    let key_2 = P256KeyPair::generate();
    actor.auth_key = Some(key_2.into());

    let mut msg = RecordsWriteBuilder::default().build().unwrap();
    actor.authorize(&mut msg).unwrap();

    assert!(dwn.process_message(&actor.did, msg).await.is_err());
}
