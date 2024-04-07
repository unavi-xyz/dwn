use aes_gcm::{
    aead::{Aead, AeadMut, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM},
    rand::{SecureRandom, SystemRandom},
};
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

impl EncryptedData {
    /// Encrypts the given data using AES-256-GCM.
    /// Returns the encrypted data and the generated key.
    pub fn encrypt_aes(data: &[u8], key: &[u8; 32]) -> Result<EncryptedData, aes_gcm::Error> {
        let key: &Key<Aes256Gcm> = key.into();

        let cipher = Aes256Gcm::new(&key);
        let iv = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&iv, data)?;

        let tag = &ciphertext[ciphertext.len() - 16..];

        Ok(Self {
            ciphertext: URL_SAFE_NO_PAD.encode(&ciphertext),
            iv: URL_SAFE_NO_PAD.encode(&iv),
            protected: Protected {
                alg: EncryptionAlgorithm::Aes256Gcm,
            },
            recipients: Vec::new(),
            tag: URL_SAFE_NO_PAD.encode(&tag),
        })
    }

    /// Decrypts the data using the given key.
    pub fn decrypt(&self, encryption: Encryption) -> Result<Vec<u8>, aes_gcm::Error> {
        match self.protected.alg {
            EncryptionAlgorithm::Aes256Gcm => match encryption {
                Encryption::Aes256Gcm(key) => self.decrypt_aes(&key),
            },
        }
    }

    fn decrypt_aes(&self, key: &[u8; 32]) -> Result<Vec<u8>, aes_gcm::Error> {
        let key: &Key<Aes256Gcm> = key.into();
        let cipher = Aes256Gcm::new(&key);

        let iv = URL_SAFE_NO_PAD.decode(self.iv.as_bytes())?;
        let tag = URL_SAFE_NO_PAD.decode(self.tag.as_bytes())?;
        let ciphertext = URL_SAFE_NO_PAD.decode(self.ciphertext.as_bytes())?;

        cipher.decrypt(iv, ciphertext)
    }
}
