use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use libipld::{ipld, pb::DagPbCodec, Cid, Ipld};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    codec::Codec,
    multihash::{Code, MultihashDigest},
    serde::to_ipld,
};
use openssl::{
    error::ErrorStack,
    rand::rand_bytes,
    symm::{Cipher, Crypter, Mode},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::EncodeError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Data {
    Base64(String),
    Encrypted(EncryptedData),
}

impl Data {
    /// Returns the CID of the data after DAG-PB encoding.
    pub fn cid(&self) -> Result<Cid, EncodeError> {
        match self {
            Data::Base64(data) => {
                let ipld = to_ipld(data)?;
                dag_pb_cid(ipld)
            }
            Data::Encrypted(data) => {
                let ipld = to_ipld(data)?;
                dag_pb_cid(ipld)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EncryptedData {
    pub protected: Protected,
    pub recipients: Vec<String>,
    pub ciphertext: String,
    pub iv: String,
    pub tag: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Protected {
    pub alg: EncryptionAlgorithm,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum EncryptionAlgorithm {
    #[serde(rename = "AES256GCM")]
    Aes256Gcm,
}

pub struct EncryptResult {
    pub data: EncryptedData,
    pub key: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum DecryptError {
    #[error(transparent)]
    OpenSSL(#[from] ErrorStack),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
}

impl EncryptedData {
    /// Encrypts the given data using AES-256-GCM.
    /// Returns the encrypted data and the generated key.
    pub fn encrypt_aes(data: &[u8]) -> Result<EncryptResult, ErrorStack> {
        let key = {
            let mut key = vec![0; 32]; // AES-256 key size
            rand_bytes(&mut key)?;
            key
        };

        let iv = {
            let mut iv = vec![0; 12]; // Recommended IV size for GCM
            rand_bytes(&mut iv)?;
            iv
        };

        let cipher = Cipher::aes_256_gcm();

        let mut encrypter = Crypter::new(cipher, Mode::Encrypt, &key, Some(&iv))?;

        let mut ciphertext = vec![0; data.len() + cipher.block_size()];
        let count = encrypter.update(data, &mut ciphertext)?;
        let rest = encrypter.finalize(&mut ciphertext[count..])?;
        ciphertext.truncate(count + rest);

        let mut tag = vec![0; 16]; // GCM tag size
        encrypter.get_tag(&mut tag)?;

        let data = Self {
            protected: Protected {
                alg: EncryptionAlgorithm::Aes256Gcm,
            },
            recipients: Vec::new(),
            ciphertext: URL_SAFE_NO_PAD.encode(&ciphertext),
            iv: URL_SAFE_NO_PAD.encode(&iv),
            tag: URL_SAFE_NO_PAD.encode(&tag),
        };

        Ok(EncryptResult { data, key })
    }

    /// Decrypts the data using the given key.
    pub fn decrypt(&self, key: &[u8]) -> Result<Vec<u8>, DecryptError> {
        match self.protected.alg {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes(key),
        }
    }

    fn decrypt_aes(&self, key: &[u8]) -> Result<Vec<u8>, DecryptError> {
        let cipher = Cipher::aes_256_gcm();

        let iv = URL_SAFE_NO_PAD.decode(self.iv.as_bytes())?;
        let tag = URL_SAFE_NO_PAD.decode(self.tag.as_bytes())?;
        let ciphertext = URL_SAFE_NO_PAD.decode(self.ciphertext.as_bytes())?;

        let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(&iv))?;
        decrypter.set_tag(&tag)?;

        let mut plaintext = vec![0; ciphertext.len() + cipher.block_size()];
        let count = decrypter.update(&ciphertext, &mut plaintext)?;
        let rest = decrypter.finalize(&mut plaintext[count..])?;
        plaintext.truncate(count + rest);

        Ok(plaintext)
    }
}

/// Returns the CID of the given IPLD after DAG-PB encoding.
fn dag_pb_cid(ipld: Ipld) -> Result<Cid, EncodeError> {
    let ipld = make_pb_compatible(ipld)?;
    let bytes = DagPbCodec.encode(&ipld)?;
    let hash = Code::Sha2_256.digest(&bytes);
    Ok(Cid::new_v1(DagPbCodec.into(), hash))
}

/// Converts the given IPLD into a format compatible with the DAG-PB codec.
fn make_pb_compatible(ipld: Ipld) -> Result<Ipld, EncodeError> {
    let mut data = None;
    let mut links = Vec::new();

    match ipld {
        Ipld::Link(cid) => {
            links.push(ipld!({
                "Hash": cid,
            }));
        }
        Ipld::List(list) => {
            for ipld in list {
                let cid = dag_pb_cid(ipld)?;

                links.push(ipld!({
                    "Hash": cid,
                }));
            }
        }
        Ipld::Map(map) => {
            for (key, value) in map {
                let cid = dag_pb_cid(value)?;

                links.push(ipld!({
                    "Hash": cid,
                    "Name": key,
                }));
            }
        }
        _ => data = Some(DagCborCodec.encode(&ipld)?),
    };

    match data {
        Some(data) => Ok(ipld!({
            "Data": data,
            "Links": links,
        })),
        None => Ok(ipld!({
            "Links": links,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_cid() {
        let data = Data::Base64("Hello, world!".to_string());
        data.cid().ok();
    }
}
