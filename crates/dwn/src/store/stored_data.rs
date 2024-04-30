use base64::{engine::general_purpose::URL_SAFE_NO_PAD, DecodeError, Engine};
use serde::{Deserialize, Serialize};

use crate::message::{Data, EncryptedData, Protected};

#[derive(Debug, Serialize, Deserialize)]
pub enum StoredData {
    Base64(Vec<u8>),
    Encrypted {
        ciphertext: Vec<u8>,
        iv: String,
        protected: Protected,
        recipients: Vec<String>,
        tag: String,
    },
}

impl StoredData {
    pub fn len(&self) -> usize {
        serde_json::to_vec(self).unwrap().len()
    }
}

impl TryFrom<Data> for StoredData {
    type Error = DecodeError;

    fn try_from(value: Data) -> Result<Self, Self::Error> {
        match value {
            Data::Base64(data) => Ok(Self::Base64(URL_SAFE_NO_PAD.decode(data.as_bytes())?)),
            Data::Encrypted(data) => Ok(Self::Encrypted {
                ciphertext: URL_SAFE_NO_PAD.decode(data.ciphertext.as_bytes())?,
                iv: data.iv,
                protected: data.protected,
                recipients: data.recipients,
                tag: data.tag,
            }),
        }
    }
}

impl From<StoredData> for Data {
    fn from(value: StoredData) -> Self {
        match value {
            StoredData::Base64(bytes) => Self::new_base64(&bytes),
            StoredData::Encrypted {
                ciphertext,
                iv,
                protected,
                recipients,
                tag,
            } => Self::Encrypted(EncryptedData {
                ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
                iv,
                protected,
                recipients,
                tag,
            }),
        }
    }
}
