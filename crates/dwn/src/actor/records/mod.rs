use base64::{DecodeError, Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dwn_core::message::{Message, data::Data};

pub mod delete;
pub mod query;
pub mod read;
pub mod write;

pub struct RecordView {
    data: Option<Vec<u8>>,
    entry: Message,
}

impl RecordView {
    fn from_entry(mut entry: Message) -> Result<Self, DecodeError> {
        let data = match entry.data.take() {
            Some(Data::Base64(encoded)) => {
                let decoded = BASE64_URL_SAFE_NO_PAD.decode(encoded)?;
                Some(decoded)
            }
            Some(Data::Encrypted(_)) => todo!(),
            None => None,
        };

        Ok(Self { data, entry })
    }

    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }
    pub fn into_data(self) -> Option<Vec<u8>> {
        self.data
    }

    pub fn entry(&self) -> &Message {
        &self.entry
    }
}
