use std::{fmt::Display, str::FromStr};

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
/// A [Decentralized Identifier](https://www.w3.org/TR/did-core/#did-syntax).
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, ':');

        if parts.next() != Some("did") {
            bail!("does not start with did")
        }

        let method_name = parts.next().ok_or(anyhow::anyhow!("no method"))?;
        let method_specific_id = parts.next().ok_or(anyhow::anyhow!("no method id"))?;

        let method_name = MethodName::from_str(method_name)?;
        let method_id = MethodId::from_str(method_specific_id)?;

        Ok(Did {
            method_name,
            method_id,
        })
    }
}

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = self.to_string();
        serializer.serialize_str(&v)
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("parse err"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MethodName(pub String);

impl FromStr for MethodName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            Ok(MethodName(s.to_string()))
        } else {
            bail!("method name must contain only lowercase letters and digits")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MethodId(pub String);

impl FromStr for MethodId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.split(':').all(is_valid_idchar) {
            Ok(MethodId(s.to_string()))
        } else {
            bail!("method id contains invalid characters")
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
