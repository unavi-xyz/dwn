use std::{fmt::Display, str::FromStr};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::{
    did::Did,
    uri::{Segment, is_segment},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidUrl {
    pub did: Did,
    /// [DID path](https://www.w3.org/TR/did-core/#path). `path-abempty` component from
    /// [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
    pub path_abempty: Option<String>,
    /// [DID query](https://www.w3.org/TR/did-core/#query). `query` component from
    /// [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
    pub query: Option<String>,
    /// [DID fragment](https://www.w3.org/TR/did-core/#fragment). `fragment` component from
    /// [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
    pub fragment: Option<String>,
}

impl Serialize for DidUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = self.to_string();
        serializer.serialize_str(&v)
    }
}

impl<'de> Deserialize<'de> for DidUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("parse err"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeDidUrl {
    pub path: RelativeDidUrlPath,
    /// [DID query](https://www.w3.org/TR/did-core/#query) ([RFC 3986 - 3.4. Query](https://www.rfc-editor.org/rfc/rfc3986#section-3.4))
    pub query: Option<String>,
    /// [DID fragment](https://www.w3.org/TR/did-core/#fragment) ([RFC 3986 - 3.5. Fragment](https://www.rfc-editor.org/rfc/rfc3986#section-3.5))
    pub fragment: Option<String>,
}

impl Display for RelativeDidUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.to_string();
        let query = match &self.query {
            Some(q) => format!("?{q}"),
            None => String::new(),
        };
        let fragment = match &self.fragment {
            Some(f) => format!("#{f}"),
            None => String::new(),
        };
        f.write_fmt(format_args!("{path}{query}{fragment}"))
    }
}

impl FromStr for RelativeDidUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (path, query, fragment) = match s.split_once('?') {
            Some((path, rest)) => match rest.split_once('#') {
                Some((query, fragment)) => (path, Some(query), Some(fragment)),
                None => (path, Some(rest), None),
            },
            None => match s.split_once('#') {
                Some((path, fragment)) => (path, None, Some(fragment)),
                None => (s, None, None),
            },
        };

        Ok(Self {
            path: RelativeDidUrlPath::from_str(path)?,
            query: query.map(|s| s.to_string()),
            fragment: fragment.map(|s| s.to_string()),
        })
    }
}

impl Serialize for RelativeDidUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = self.to_string();
        serializer.serialize_str(&v)
    }
}

impl<'de> Deserialize<'de> for RelativeDidUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("parse err"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelativeDidUrlPath {
    /// Absolute-path reference. `path-absolute` from [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.3)
    Absolute(String),
    /// Relative-path reference. `path-noscheme` from [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.3)
    NoScheme(String),
    /// Empty path. `path-empty` from [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.3)
    Empty,
}

impl Display for RelativeDidUrlPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            Self::Absolute(s) | Self::NoScheme(s) => s.as_str(),
            Self::Empty => "",
        };
        f.write_str(data)
    }
}

impl FromStr for RelativeDidUrlPath {
    type Err = anyhow::Error;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        if path.is_empty() {
            return Ok(Self::Empty);
        }
        if path.starts_with('/') {
            // path-absolute = "/" [ segment-nz *( "/" segment ) ]
            if path.len() >= 2 && path.chars().nth(1) == Some('/') {
                bail!("double slash at start")
            }

            if !path
                .split('/')
                .skip(1)
                .all(|v| is_segment(v, Segment::Base))
            {
                bail!("invalid segment")
            }

            Ok(Self::Absolute(path.to_string()))
        } else {
            // path-noscheme = segment-nz-nc *( "/" segment )
            if !path.split('/').all(|v| is_segment(v, Segment::NzNc)) {
                bail!("invalid segment")
            }

            Ok(Self::NoScheme(path.to_string()))
        }
    }
}

impl DidUrl {
    /// Attempts to convert the [DidUrl] into a [RelativeDidUrl].
    pub fn to_relative(&self) -> Option<RelativeDidUrl> {
        Some(RelativeDidUrl {
            path: match RelativeDidUrlPath::from_str(&self.path_abempty.clone().unwrap_or_default())
            {
                Ok(v) => v,
                Err(_) => return None,
            },
            fragment: self.fragment.clone(),
            query: self.query.clone(),
        })
    }
}

impl Display for DidUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut url = self.did.to_string();

        if let Some(ref path) = self.path_abempty {
            url.push_str(path);
        }

        if let Some(ref query) = self.query {
            url.push('?');
            url.push_str(query);
        }

        if let Some(ref fragment) = self.fragment {
            url.push('#');
            url.push_str(fragment);
        }

        f.write_str(&url)
    }
}

impl FromStr for DidUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let did_str = s.find(['/', '?', '#']).map(|pos| &s[..pos]).unwrap_or(s);

        let did = Did::from_str(did_str)?;

        let mut path = String::new();
        let mut query = None;
        let mut fragment = None;

        let mut rest = s.strip_prefix(did_str).unwrap();
        if let Some((before_fragment, frag)) = rest.split_once('#') {
            fragment = Some(frag.to_string());
            rest = before_fragment;
        }

        if let Some((before_query, qry)) = rest.split_once('?') {
            query = Some(qry.to_string());
            rest = before_query;
        }

        path.push_str(rest);

        // path-abempty  = *( "/" segment )
        let path_abempty = if path.is_empty() {
            None
        } else {
            if !path.starts_with('/') {
                bail!("path_abempty does not start with slash")
            }

            if !path.split('/').all(|v| is_segment(v, Segment::Base)) {
                bail!("invalid path_abempty segment")
            }

            Some(path)
        };

        Ok(DidUrl {
            did,
            path_abempty,
            query,
            fragment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: Some("/path/to/resource".to_string()),
            query: Some("key=value".to_string()),
            fragment: Some("section".to_string()),
        };

        let serialized = did_url.to_string();
        assert_eq!(
            serialized,
            "did:example:123/path/to/resource?key=value#section"
        );

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_no_path() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: None,
            query: Some("key=value".to_string()),
            fragment: Some("section".to_string()),
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123?key=value#section");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_no_query() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: Some("/path/to/resource".to_string()),
            query: None,
            fragment: Some("section".to_string()),
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123/path/to/resource#section");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_no_fragment() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: Some("/path/to/resource".to_string()),
            query: Some("key=value".to_string()),
            fragment: None,
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123/path/to/resource?key=value");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_did_plain() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: None,
            query: None,
            fragment: None,
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_compound_query() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: None,
            query: Some("a=1&b=2".to_string()),
            fragment: None,
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123?a=1&b=2");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_dwn_ref() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: None,
            query: Some("service=dwn&relativeRef=/records/abc123".to_string()),
            fragment: None,
        };

        let serialized = did_url.to_string();
        assert_eq!(
            serialized,
            "did:example:123?service=dwn&relativeRef=/records/abc123"
        );

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }
}
