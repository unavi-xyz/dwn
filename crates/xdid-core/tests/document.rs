use serde_json::Value;
use xdid_core::{
    did::{Did, MethodId, MethodName},
    did_url::{DidUrl, RelativeDidUrl, RelativeDidUrlPath},
    document::{Document, VerificationMethod, VerificationMethodMap},
};

#[test]
fn test_document_serde() {
    let did = Did {
        method_name: MethodName("web".to_string()),
        method_id: MethodId("localhost%3A4000".to_string()),
    };

    let doc = Document {
        id: did.clone(),
        also_known_as: None,
        assertion_method: Some(vec![VerificationMethod::RelativeUrl(RelativeDidUrl {
            path: RelativeDidUrlPath::Empty,
            query: None,
            fragment: Some("owner".to_string()),
        })]),
        authentication: None,
        capability_delegation: Some(vec![VerificationMethod::Url(DidUrl {
            did: did.clone(),
            path_abempty: String::new(),
            query: Some("test-query".to_string()),
            fragment: Some("owner".to_string()),
        })]),
        capability_invocation: None,
        controller: None,
        key_agreement: None,
        service: None,
        verification_method: Some(vec![VerificationMethodMap {
            id: DidUrl {
                did: did.clone(),
                fragment: Some("owner".to_string()),
                query: None,
                path_abempty: String::new(),
            },
            controller: did,
            typ: "JsonWebKey2020".to_string(),
            public_key_multibase: None,
            public_key_jwk: None,
        }]),
    };

    let doc_val = serde_json::to_value(&doc).unwrap();
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());

    const EXPECTED_RAW: &[u8] = include_bytes!("./document-expected.json");
    let expected_val: Value = serde_json::from_slice(EXPECTED_RAW).unwrap();
    assert_eq!(doc_val, expected_val);

    let expected_doc: Document = serde_json::from_slice(EXPECTED_RAW).unwrap();
    assert_eq!(doc, expected_doc);
}
