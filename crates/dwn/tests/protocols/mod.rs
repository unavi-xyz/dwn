use dwn_core::message::{Version, descriptor::ProtocolDefinition, mime::TEXT_PLAIN};
use serde_json::json;
use tracing_test::traced_test;

use crate::utils::init_dwn;

mod create;

#[tokio::test]
#[traced_test]
async fn test_protocol_configure_query() {
    let (alice, ..) = init_dwn();

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
    let path = "my-value".to_string();

    let record_id = alice
        .write()
        .protocol(definition.protocol.clone(), version.clone(), path.clone())
        .data(TEXT_PLAIN, data.clone())
        .target(&alice.did)
        .process()
        .await
        .unwrap();

    let found = alice
        .query()
        .protocol(definition.protocol)
        .protocol_version(version)
        .protocol_path(path)
        .process()
        .await
        .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].entry().record_id, record_id);
}
