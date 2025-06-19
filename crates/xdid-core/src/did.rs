use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::did_url::ParseError;

/// [DID](https://www.w3.org/TR/did-core/#did-syntax).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Did {
    pub method_name: MethodName,
    pub method_id: MethodId,
}

impl Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did:{}:{}", self.method_name.0, self.method_id.0)
    }
}

impl FromStr for Did {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, ':');

        if parts.next() != Some("did") {
            return Err(ParseError);
        }

        let method_name = parts.next().ok_or(ParseError)?;
        let method_specific_id = parts.next().ok_or(ParseError)?;

        let method_name = MethodName::from_str(method_name).map_err(|_| ParseError)?;
        let method_id = MethodId::from_str(method_specific_id).map_err(|_| ParseError)?;

        Ok(Did {
            method_name,
            method_id,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MethodName(pub String);

impl FromStr for MethodName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            Ok(MethodName(s.to_string()))
        } else {
            Err("Method name must contain only lowercase letters and digits".into())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MethodId(pub String);

impl FromStr for MethodId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.split(':').all(is_valid_idchar) {
            Ok(MethodId(s.to_string()))
        } else {
            Err("Method-specific ID contains invalid characters".into())
        }
    }
}

fn is_valid_idchar(s: &str) -> bool {
    s.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == '.'
            || c == '-'
            || c == '_'
            || c == '%'
            || c.is_ascii_hexdigit()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_example() {
        let did = Did {
            method_name: MethodName("example".to_string()),
            method_id: MethodId("1234-5678-abcdef".to_string()),
        };

        let serialized = did.to_string();
        assert_eq!(serialized, "did:example:1234-5678-abcdef");

        let deserialized = Did::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did);
    }
}
