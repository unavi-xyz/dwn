use std::sync::Arc;

use dwn::{
    actor::Actor,
    message::descriptor::{
        protocols::{
            Action, ActionCan, ActionWho, ProtocolDefinition, ProtocolStructure, ProtocolType,
            ProtocolsFilter,
        },
        Descriptor,
    },
    store::SurrealStore,
    DWN,
};
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_configure_protocol() {
    let store = SurrealStore::new().await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let actor = Actor::new_did_key(dwn).unwrap();

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
        .register_protocol(definition.clone())
        .protocol_version("0.1.0".to_string())
        .process()
        .await
        .unwrap();
    assert_eq!(register.status.code, 200);

    let filter = ProtocolsFilter {
        protocol: "test-protocol".to_string(),
        versions: vec!["0.1.0".to_string()],
    };

    let query = actor.query_protocols(filter).process().await.unwrap();
    assert_eq!(query.status.code, 200);
    assert_eq!(query.entries.len(), 1);

    let descriptor = match &query.entries[0].descriptor {
        Descriptor::ProtocolsConfigure(descriptor) => descriptor,
        _ => panic!("unexpected descriptor"),
    };

    assert_eq!(
        descriptor.definition.as_ref().unwrap().protocol,
        "test-protocol"
    );
    assert_eq!(descriptor.protocol_version, Some("0.1.0".to_string()));
}
