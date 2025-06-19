use multibase::Base;
use thiserror::Error;
use xdid_core::did::Did;

use crate::keys::{KeyParser, PublicKey};

pub struct DidKeyParser {
    parsers: Vec<Box<dyn KeyParser>>,
}

impl Default for DidKeyParser {
    fn default() -> Self {
        let parsers: Vec<Box<dyn KeyParser>> = vec![
            #[cfg(feature = "p256")]
            Box::new(crate::keys::p256::P256KeyParser),
            #[cfg(feature = "p384")]
            Box::new(crate::keys::p384::P384KeyParser),
        ];

        Self { parsers }
    }
}

impl DidKeyParser {
    pub fn parse(&self, did: &Did) -> Result<Box<dyn PublicKey>, ParseError> {
        let (base, inner) = multibase::decode(&did.method_id.0)?;
        debug_assert_eq!(base, Base::Base58Btc);

        for parser in self.parsers.iter() {
            let code = parser.codec().code();
            if let Some(bytes) = inner.strip_prefix(code.as_slice()) {
                return Ok(parser.parse(bytes.to_vec()));
            }
        }

        Err(ParseError::CodecNotSupported)
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to decode multibase: {0}")]
    Decode(#[from] multibase::Error),
    #[error("codec not supported")]
    CodecNotSupported,
}
