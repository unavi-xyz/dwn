use dwn_core::message::{Version, descriptor::ProtocolDefinition, mime::TEXT_PLAIN};
use serde_json::json;
use tracing_test::traced_test;

use crate::utils::init_dwn;

#[tokio::test]
#[traced_test]
async fn test_protocol_anyone_create() {
    let (alice, bob, dwn) = init_dwn();

    let raw_definition = json!({
        "protocol": "my-protocol",
        "published": true,
        "types": {
            "my-value": {
                "dataFormat": ["text/plain"],
            },
            "non-value": {
                "dataFormat": ["text/plain"],
            }
        },
        "structure": {
            "my-value": {
                "$actions": [{
                    "who": "anyone",
                    "can": ["create"],
                }]
            },
            "non-value": {}
        }
    });
    let definition = serde_json::from_value::<ProtocolDefinition>(raw_definition).unwrap();
    let version = Version::new(1, 2, 3);

    alice
        .configure_protocol(version.clone(), definition.clone())
        .process()
        .await
        .unwrap();

    let data = "Hello, world!".as_bytes().to_vec();

    let record_id = bob
        .write()
        .protocol(
            definition.protocol.clone(),
            version.clone(),
            "my-value".to_string(),
        )
        .data(TEXT_PLAIN, data.clone())
        .target(&alice.did)
        .process()
        .await
        .unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &alice.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.initial_entry.record_id, record_id);
    assert_eq!(found.latest_entry.record_id, record_id);

    let res = alice
        .write()
        .protocol(definition.protocol, version, "non-value".to_string())
        .data(TEXT_PLAIN, data)
        .process()
        .await;
    assert!(res.is_err());
}

#[tokio::test]
#[traced_test]
async fn test_protocol_author_update() {
    let (alice, bob, dwn) = init_dwn();

    let raw_definition = json!({
        "protocol": "my-protocol",
        "published": true,
        "types": {
            "my-value": {
                "dataFormat": ["text/plain"],
            }
        },
        "structure": {
            "my-value": {
                "$actions": [
                    {
                        "who": "anyone",
                        "can": ["create"],
                    },
                    {
                        "who": "author",
                        "can": ["update"],
                    },
                ]
            }
        }
    });
    let definition = serde_json::from_value::<ProtocolDefinition>(raw_definition).unwrap();
    let version = Version::new(1, 2, 3);

    alice
        .configure_protocol(version.clone(), definition.clone())
        .process()
        .await
        .unwrap();

    let data_1 = "Hello, world!".as_bytes().to_vec();
    let data_2 = "Goodbye, world!".as_bytes().to_vec();

    let record_id = bob
        .write()
        .protocol(
            definition.protocol.clone(),
            version.clone(),
            "my-value".to_string(),
        )
        .data(TEXT_PLAIN, data_1.clone())
        .target(&alice.did)
        .process()
        .await
        .unwrap();

    bob.write()
        .record_id(record_id.clone())
        .protocol(
            definition.protocol.clone(),
            version.clone(),
            "my-value".to_string(),
        )
        .data(TEXT_PLAIN, data_2.clone())
        .target(&alice.did)
        .process()
        .await
        .unwrap();

    let found = dwn
        .record_store
        .read(dwn.data_store.as_ref(), &alice.did, &record_id)
        .unwrap()
        .unwrap();
    assert_eq!(found.initial_entry.record_id, record_id);
    assert_eq!(found.latest_entry.record_id, record_id);
    assert_ne!(found.initial_entry.data, found.latest_entry.data);

    let res = alice
        .write()
        .record_id(record_id.clone())
        .protocol(
            definition.protocol.clone(),
            version.clone(),
            "my-value".to_string(),
        )
        .data(TEXT_PLAIN, data_2.clone())
        .target(&alice.did)
        .process()
        .await;
    assert!(res.is_err())
}
