use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use openssl::{
    error::ErrorStack,
    rand::rand_bytes,
    symm::{Cipher, Crypter, Mode},
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

#[derive(Debug, Error)]
pub enum DecryptError {
    #[error("Decryption key mismatch: expected {0:?}")]
    KeyMismatch(EncryptionAlgorithm),
    #[error(transparent)]
    OpenSSL(#[from] ErrorStack),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
}

impl EncryptedData {
    /// Encrypts the given data using AES-256-GCM.
    /// Returns the encrypted data and the generated key.
    pub fn encrypt_aes(data: &[u8], key: &[u8]) -> Result<EncryptedData, ErrorStack> {
        let iv = {
            let mut iv = vec![0; 12]; // Recommended IV size for GCM
            rand_bytes(&mut iv)?;
            iv
        };

        let cipher = Cipher::aes_256_gcm();

        let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(&iv))?;

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

        Ok(data)
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
