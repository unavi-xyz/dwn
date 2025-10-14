use jose_jwk::Jwk;
use p256::{
    elliptic_curve::{rand_core::OsRng, zeroize::Zeroizing},
    pkcs8::{DecodePrivateKey, LineEnding},
};
use p384::{
    SecretKey,
    elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
    pkcs8::EncodePrivateKey,
};
use ring::{
    rand::SystemRandom,
    signature::{ECDSA_P384_SHA384_ASN1_SIGNING, EcdsaKeyPair},
};

use super::{DidKeyPair, KeyParser, Multicodec, PublicKey, SignError, Signer, WithMulticodec};

#[derive(Clone, PartialEq, Eq)]
pub struct P384KeyPair(SecretKey);

impl DidKeyPair for P384KeyPair {
    fn generate() -> Self {
        let mut rng = OsRng;
        let secret = SecretKey::random(&mut rng);
        Self(secret)
    }

    fn public(&self) -> impl PublicKey {
        P384PublicKey(self.0.public_key())
    }

    fn to_pkcs8_pem(&self) -> anyhow::Result<Zeroizing<String>> {
        let pem = self.0.to_pkcs8_pem(LineEnding::LF)?;
        Ok(pem)
    }
    fn from_pkcs8_pem(pem: &str) -> anyhow::Result<Self> {
        let key = SecretKey::from_pkcs8_pem(pem)?;
        Ok(Self(key))
    }
}

impl Signer for P384KeyPair {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, SignError> {
        let rng = SystemRandom::new();

        let signer = EcdsaKeyPair::from_private_key_and_public_key(
            &ECDSA_P384_SHA384_ASN1_SIGNING,
            &self.0.to_bytes(),
            &self.0.public_key().to_sec1_bytes(),
            &rng,
        )
        .unwrap();

        signer
            .sign(&rng, message)
            .map(|v| v.as_ref().to_vec())
            .map_err(|_| SignError::SigningFailed)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct P384PublicKey(p384::PublicKey);

impl PublicKey for P384PublicKey {
    fn to_sec1_bytes(&self) -> Box<[u8]> {
        self.0.to_sec1_bytes()
    }
    fn to_encoded_point_bytes(&self) -> Box<[u8]> {
        self.0.to_encoded_point(true).as_bytes().into()
    }

    fn to_jwk(&self) -> Jwk {
        let jwk_str = self.0.to_jwk_string();
        serde_json::from_str(&jwk_str).unwrap()
    }
}

impl WithMulticodec for P384PublicKey {
    fn codec(&self) -> Box<dyn Multicodec> {
        Box::new(P384Codec)
    }
}

pub(crate) struct P384KeyParser;

impl KeyParser for P384KeyParser {
    fn parse(&self, public_key: Vec<u8>) -> Box<dyn PublicKey> {
        let point = p384::EncodedPoint::from_bytes(public_key).unwrap();
        let key = p384::PublicKey::from_encoded_point(&point).unwrap();
        Box::new(P384PublicKey(key))
    }
}

impl WithMulticodec for P384KeyParser {
    fn codec(&self) -> Box<dyn Multicodec> {
        Box::new(P384Codec)
    }
}

struct P384Codec;

impl Multicodec for P384Codec {
    fn code_u64(&self) -> u64 {
        0x1201
    }
}

#[cfg(test)]
mod tests {
    use ring::signature::{ECDSA_P384_SHA384_ASN1, VerificationAlgorithm};

    use crate::parser::DidKeyParser;

    use super::*;

    #[test]
    fn test_display() {
        let pair = P384KeyPair::generate();
        let did = pair.public().to_did();

        let did_str = did.to_string();
        println!("{}", did_str);
        assert!(did_str.starts_with("did:key:z82"));
    }

    #[test]
    fn test_jwk() {
        let pair = P384KeyPair::generate();
        let _ = pair.public().to_jwk();
    }

    #[test]
    fn test_parse() {
        let pair = P384KeyPair::generate();
        let did = pair.public().to_did();

        let parser = DidKeyParser::default();
        let _ = parser.parse(&did).unwrap();
    }

    #[test]
    fn test_sign() {
        let pair = P384KeyPair::generate();

        let msg = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let signature = pair.sign(&msg).unwrap();

        assert!(
            ECDSA_P384_SHA384_ASN1
                .verify(
                    pair.public().to_sec1_bytes().as_ref().into(),
                    msg.as_slice().into(),
                    signature.as_slice().into()
                )
                .is_ok()
        );
    }
}
