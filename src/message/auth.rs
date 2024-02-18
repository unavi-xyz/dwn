use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use didkit::ssi::jwk::Algorithm;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AuthPayload {
    #[serde(rename = "attestationCid")]
    pub attestation_cid: Option<String>,
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
    #[serde(rename = "permissionsGrantCid")]
    pub permissions_grant_cid: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JWS<T> {
    pub payload: T,
    pub signatures: Vec<SignatureEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SignatureEntry {
    pub protected: Protected,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Protected {
    /// Algorithm used to verify the signature
    pub alg: Algorithm,
    /// DID URL of the key used to verify the signature
    pub kid: String,
}

#[derive(Deserialize, Serialize)]
struct ProtectedJson {
    pub alg: Algorithm,
    pub kid: String,
}

impl<'de> Deserialize<'de> for Protected {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(serde::de::Error::custom)?;
        let json =
            serde_json::from_slice::<ProtectedJson>(&decoded).map_err(serde::de::Error::custom)?;
        Ok(Protected {
            alg: json.alg,
            kid: json.kid,
        })
    }
}

impl Serialize for Protected {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let json = ProtectedJson {
            alg: self.alg,
            kid: self.kid.clone(),
        };
        let json_string = serde_json::to_string(&json).map_err(serde::ser::Error::custom)?;
        let encoded = URL_SAFE_NO_PAD.encode(json_string);
        serializer.serialize_str(&encoded)
    }
}
