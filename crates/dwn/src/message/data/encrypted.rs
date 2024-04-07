use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::actor::records::Encryption;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub iv: String,
    pub protected: Protected,
    pub recipients: Vec<String>,
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
    #[error("Decryption key mismatch: expected {0:?}")]
    KeyMismatch(EncryptionAlgorithm),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error("Decryption failed")]
    Decryption,
}

impl EncryptedData {
    /// Encrypts the given data using AES-256-GCM.
    /// Returns the encrypted data and the generated key.
    pub fn encrypt_aes(data: &[u8], key: &[u8]) -> Result<EncryptedData, aes_gcm::Error> {
        let key: &Key<Aes256Gcm> = key.into();

        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, data)?;

        let (ciphertext, tag) = ciphertext.split_at(ciphertext.len() - 16);

        Ok(Self {
            ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
            iv: URL_SAFE_NO_PAD.encode(nonce),
            protected: Protected {
                alg: EncryptionAlgorithm::Aes256Gcm,
            },
            recipients: Vec::new(),
            tag: URL_SAFE_NO_PAD.encode(tag),
        })
    }

    /// Decrypts the data using the given key.
    pub fn decrypt(&self, encryption: Encryption) -> Result<Vec<u8>, DecryptError> {
        match self.protected.alg {
            EncryptionAlgorithm::Aes256Gcm => match encryption {
                Encryption::Aes256Gcm(key) => self.decrypt_aes(&key),
            },
        }
    }

    fn decrypt_aes(&self, key: &[u8]) -> Result<Vec<u8>, DecryptError> {
        let key: &Key<Aes256Gcm> = key.into();
        let cipher = Aes256Gcm::new(key);

        let iv = URL_SAFE_NO_PAD.decode(self.iv.as_bytes())?;
        let tag = URL_SAFE_NO_PAD.decode(self.tag.as_bytes())?;
        let mut ciphertext = URL_SAFE_NO_PAD.decode(self.ciphertext.as_bytes())?;
        ciphertext.extend(tag);

        let nonce = Nonce::from_slice(&iv);

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| DecryptError::Decryption)
    }
}
