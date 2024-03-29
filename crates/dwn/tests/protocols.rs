use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::descriptor::{
        Action, ActionCan, ActionWho, ProtocolDefinition, ProtocolStructure, ProtocolType,
    },
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_protocols() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let actor = Actor::new_did_key(dwn).unwrap();

    // Create a new protocol.
    let mut definition = ProtocolDefinition {
        protocol: "test-protocol".to_string(),
        published: true,
        ..Default::default()
    };

    definition.types.insert(
        "test".to_string(),
        ProtocolType {
            data_format: vec!["application/json".to_string()],
            ..Default::default()
        },
    );

    let mut structure = ProtocolStructure::default();
    structure.actions.push(Action {
        who: ActionWho::Author,
        of: None,
        can: ActionCan::Write,
    });
    definition.structure.insert("test".to_string(), structure);

    let register = actor
        .register_protocol(definition)
        .protocol_version("0.1.0".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);
}