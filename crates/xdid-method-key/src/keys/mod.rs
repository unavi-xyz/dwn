use jose_jwk::Jwk;
use multibase::Base;
use thiserror::Error;
use xdid_core::did::{Did, MethodId, MethodName};

use crate::NAME;

#[cfg(feature = "p256")]
pub mod p256;
#[cfg(feature = "p384")]
pub mod p384;

pub trait Signer {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, SignError>;
}

pub trait DidKeyPair: Signer {
    /// Generate a new pair of keys.
    fn generate() -> Self;

    fn public(&self) -> impl PublicKey;

    /// Raw public key bytes.
    fn public_bytes(&self) -> Box<[u8]>;
    /// Raw secret key bytes.
    fn secret_bytes(&self) -> Box<[u8]>;
}

#[derive(Error, Debug)]
pub enum SignError {
    #[error("signing failed")]
    SigningFailed,
}

pub trait PublicKey: WithMulticodec {
    /// Read the public key as DID-ready bytes.
    /// This may be different from the raw public key bytes, as some algorithms
    /// require compression.
    /// https://w3c-ccg.github.io/did-method-key/#signature-method-creation-algorithm
    fn as_did_bytes(&self) -> Box<[u8]>;
    fn to_jwk(&self) -> Jwk;

    fn to_did(&self) -> Did {
        let bytes = self.as_did_bytes();
        let code = self.codec().code();

        let mut inner = Vec::with_capacity(code.len() + bytes.len());
        inner.extend(code);
        inner.extend(bytes);

        let id = multibase::encode(Base::Base58Btc, inner);

        Did {
            method_name: MethodName(NAME.to_string()),
            method_id: MethodId(id),
        }
    }
}

pub trait Multicodec {
    fn code_u64(&self) -> u64;
    fn code(&self) -> Vec<u8> {
        let mut buffer = unsigned_varint::encode::u64_buffer();
        unsigned_varint::encode::u64(self.code_u64(), &mut buffer).to_vec()
    }
}

pub trait WithMulticodec {
    fn codec(&self) -> Box<dyn Multicodec>;
}

pub trait KeyParser: WithMulticodec {
    fn parse(&self, public_key: Vec<u8>) -> Box<dyn PublicKey>;
}
