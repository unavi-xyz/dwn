use std::{collections::HashMap, sync::Arc};

use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::protocols::{
        Action, ActionCan, ActionWho, ProtocolDefinition, ProtocolStructure, StructureType,
    },
    store::SurrealStore,
    DWN,
};
use iana_media_types::Application;
use semver::Version;
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;

const PROTOCOL: &str = "post-protocol";

#[tokio::test]
#[traced_test]
async fn test_child_anyone_write() {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let alice = Actor::new_did_key(dwn.clone()).unwrap();
    let bob = Actor::new_did_key(dwn.clone()).unwrap();

    // Alice registers protocol.
    let mut definition = ProtocolDefinition {
        protocol: PROTOCOL.to_string(),
        published: true,
        ..Default::default()
    };

    definition
        .types
        .insert("post".to_string(), StructureType::default());

    definition
        .types
        .insert("comment".to_string(), StructureType::default());

    let mut post_children = HashMap::new();

    post_children.insert(
        "comment".to_string(),
        ProtocolStructure {
            actions: vec![Action {
                who: ActionWho::Anyone,
                of: None,
                can: ActionCan::Write,
            }],
            ..Default::default()
        },
    );

    definition.structure.insert(
        "post".to_string(),
        ProtocolStructure {
            children: post_children,
            ..Default::default()
        },
    );

    let register = alice
        .register_protocol(definition.clone())
        .protocol_version(Version::new(0, 1, 0))
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    // Alice creates a post.
    let post_data = "Hello post".as_bytes().to_vec();
    let post = alice
        .create_record()
        .data(post_data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            "post".to_string(),
        )
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(post.reply.status.code, 200);

    // Alice can create a comment.
    let comment_data = "Hello comment".as_bytes().to_vec();
    let comment = alice
        .create_record()
        .data(comment_data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            "comment".to_string(),
        )
        .parent_context_id(post.context_id.clone().unwrap())
        .published(true)
        .process()
        .await
        .unwrap();
    assert_eq!(comment.reply.status.code, 200);

    // Comments must specify context id.
    let comment = alice
        .create_record()
        .data(comment_data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            "comment".to_string(),
        )
        .published(true)
        .process()
        .await;
    assert!(comment.is_err());

    // Bob can comment.
    let comment = bob
        .create_record()
        .data(comment_data.clone())
        .data_format(Application::Json.into())
        .protocol(
            PROTOCOL.to_string(),
            Version::new(0, 1, 0),
            "comment".to_string(),
        )
        .published(true)
        .target(alice.did.clone())
        .parent_context_id(post.context_id.unwrap())
        .process()
        .await
        .unwrap();
    assert_eq!(comment.reply.status.code, 200);
}
