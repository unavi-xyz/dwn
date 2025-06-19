use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{
    did::Did,
    uri::{is_segment, Segment},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DidUrl {
    pub did: Did,
    /// [DID path](https://www.w3.org/TR/did-core/#path). `path-abempty` component from
    /// [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
    pub path_abempty: String,
    /// [DID query](https://www.w3.org/TR/did-core/#query). `query` component from
    /// [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
    pub query: Option<String>,
    /// [DID fragment](https://www.w3.org/TR/did-core/#fragment). `fragment` component from
    /// [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
    pub fragment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RelativeDidUrl {
    pub path: RelativeDidUrlPath,
    /// [DID query](https://www.w3.org/TR/did-core/#query) ([RFC 3986 - 3.4. Query](https://www.rfc-editor.org/rfc/rfc3986#section-3.4))
    pub query: Option<String>,
    /// [DID fragment](https://www.w3.org/TR/did-core/#fragment) ([RFC 3986 - 3.5. Fragment](https://www.rfc-editor.org/rfc/rfc3986#section-3.5))
    pub fragment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum RelativeDidUrlPath {
    /// Absolute-path reference. `path-absolute` from [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.3)
    Absolute(String),
    /// Relative-path reference. `path-noscheme` from [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.3)
    NoScheme(String),
    /// Empty path. `path-empty` from [RFC 3986](https://tools.ietf.org/html/rfc3986#section-3.3)
    Empty,
}

impl FromStr for RelativeDidUrlPath {
    type Err = ParseError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        if path.is_empty() {
            return Ok(Self::Empty);
        }
        if path.starts_with('/') {
            // path-absolute = "/" [ segment-nz *( "/" segment ) ]
            if path.len() >= 2 && path.chars().nth(1) == Some('/') {
                return Err(ParseError);
            }

            if !path
                .split('/')
                .skip(1)
                .all(|v| is_segment(v, Segment::Base))
            {
                return Err(ParseError);
            }

            Ok(Self::Absolute(path.to_string()))
        } else {
            // path-noscheme = segment-nz-nc *( "/" segment )
            if !path.split('/').all(|v| is_segment(v, Segment::NzNc)) {
                return Err(ParseError);
            }

            Ok(Self::NoScheme(path.to_string()))
        }
    }
}

impl DidUrl {
    /// Attempts to convert the [DidUrl] into a [RelativeDidUrl].
    pub fn to_relative(&self) -> Option<RelativeDidUrl> {
        Some(RelativeDidUrl {
            path: match RelativeDidUrlPath::from_str(&self.path_abempty) {
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
        let mut url = format!("{}{}", self.did, self.path_abempty);

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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (did_str, _) = s.split_once('/').unwrap_or_else(|| {
            s.split_once('?')
                .unwrap_or_else(|| s.split_once('#').unwrap_or((s, "")))
        });

        let did = Did::from_str(did_str).map_err(|_| ParseError)?;

        let mut path_abempty = String::new();
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

        path_abempty.push_str(rest);

        // path-abempty  = *( "/" segment )
        if !path_abempty.is_empty() {
            if !path_abempty.starts_with('/') {
                return Err(ParseError);
            }

            if !path_abempty
                .split('/')
                .all(|v| is_segment(v, Segment::Base))
            {
                return Err(ParseError);
            }
        }

        Ok(DidUrl {
            did,
            path_abempty,
            query,
            fragment,
        })
    }
}

#[derive(Debug)]
pub struct ParseError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_url_full() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: "/path/to/resource".to_string(),
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
    fn test_did_url_no_path() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: "".to_string(),
            query: Some("key=value".to_string()),
            fragment: Some("section".to_string()),
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123?key=value#section");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_did_url_no_query() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: "/path/to/resource".to_string(),
            query: None,
            fragment: Some("section".to_string()),
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123/path/to/resource#section");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_did_url_no_fragment() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: "/path/to/resource".to_string(),
            query: Some("key=value".to_string()),
            fragment: None,
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123/path/to/resource?key=value");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }

    #[test]
    fn test_did_url_none() {
        let did_url = DidUrl {
            did: Did::from_str("did:example:123").unwrap(),
            path_abempty: "".to_string(),
            query: None,
            fragment: None,
        };

        let serialized = did_url.to_string();
        assert_eq!(serialized, "did:example:123");

        let deserialized = DidUrl::from_str(&serialized).expect("deserialize failed");
        assert_eq!(deserialized, did_url);
    }
}
