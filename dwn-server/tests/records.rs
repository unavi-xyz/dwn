use didkit::{ssi::jwk::Algorithm, Source, DIDURL, DID_METHODS};
use dwn::request::{
    descriptor::records::RecordsWrite,
    message::{AuthPayload, Authorization, Message},
    RequestBody,
};
use dwn_test_utils::{expect_status, spawn_server};
use reqwest::StatusCode;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn records_write() {
    let port = spawn_server().await;

    let mut msg = Message::new(RecordsWrite::default());

    // Require authorization
    {
        let body = RequestBody {
            messages: vec![msg.clone()],
        };

        expect_status(body, port, StatusCode::UNAUTHORIZED).await;
    }

    let payload = AuthPayload {
        descriptor_cid: msg.descriptor.cid().to_string(),
        attestation_cid: None,
        permissions_grant_cid: None,
    };

    let key = didkit::JWK::generate_ed25519().expect("failed to generate key");
    let did = DID_METHODS
        .get_method("did:key")
        .expect("did:key method not found")
        .generate(&Source::Key(&key))
        .expect("failed to generate did");
    let mut key_url = DIDURL::try_from(did.clone()).expect("failed to parse did url");
    let did_hash = did.split(':').nth(2).expect("failed to get did body");
    key_url.fragment = Some(did_hash.to_string());

    let auth = Authorization::encode(Algorithm::EdDSA, &payload, &key, &key_url)
        .await
        .expect("failed to encode authorization");
    msg.authorization = Some(auth);

    // Valid message
    {
        let body = RequestBody {
            messages: vec![msg],
        };

        expect_status(body, port, StatusCode::OK).await;
    }
}
