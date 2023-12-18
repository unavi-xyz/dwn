use std::fmt::Display;

pub use iana_media_types as media_types;
// use libipld_core::{codec::Codec, ipld::Ipld};
// use libipld_pb::DagPbCodec;
use serde::{Deserialize, Serialize};

use crate::util::cid_from_bytes;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "recordId")]
    pub record_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    pub descriptor: Descriptor,
}

pub struct MessageBuilder {
    pub data: Option<String>,
    pub descriptor: DescriptorBuilder,
}

impl MessageBuilder {
    pub fn build(&self) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(Message {
            record_id: self.generate_record_id()?,
            data: self.data.clone(), // TODO: bas64 encode data
            descriptor: self.descriptor.build()?,
        })
    }

    pub fn generate_record_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let descriptor = self.descriptor.build()?;
        let generator = RecordIdGenerator::try_from(&descriptor)?;
        let record_id = generator.generate_cid()?;
        Ok(record_id)
    }
}

#[derive(Serialize)]
pub struct RecordIdGenerator {
    #[serde(rename = "descriptorCid")]
    pub descriptor_cid: String,
}

impl RecordIdGenerator {
    /// Generates the CID of this struct after DAG-CBOR serialization.
    pub fn generate_cid(&self) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        let cid = cid_from_bytes(&bytes);
        Ok(cid.to_string())
    }
}

impl TryFrom<&Descriptor> for RecordIdGenerator {
    type Error = Box<dyn std::error::Error>;

    fn try_from(descriptor: &Descriptor) -> Result<Self, Self::Error> {
        let serialized = serde_ipld_dagcbor::to_vec(descriptor)?;
        let descriptor_cid = cid_from_bytes(&serialized).to_string();
        Ok(RecordIdGenerator { descriptor_cid })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Descriptor {
    pub interface: Interface,
    pub method: Method,
    #[serde(rename = "dataCid", skip_serializing_if = "Option::is_none")]
    pub data_cid: Option<String>,
    #[serde(rename = "dataFormat", skip_serializing_if = "Option::is_none")]
    pub data_format: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Interface {
    /// For feature detection, the spec only lists a method, no interface.
    /// But the spec also says that an interface MUST exist in the descriptor.
    /// So we use this interface for feature detection, even though it doesn't exist in the spec.
    /// https://identity.foundation/decentralized-web-node/spec/#read
    FeatureDetection,
    Records,
    Protocols,
    Permissions,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Method {
    FeatureDetectionRead,

    RecordsRead,
    RecordsQuery,
    RecordsWrite,
    RecordsCommit,
    RecordsDelete,

    ProtocolsConfigure,
    ProtocolsQuery,

    PermissionsRequest,
    PermissionsGrant,
    PermissionsRevoke,
    PermissionsQuery,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

pub enum DataFormat {
    /// JSON Web Token formatted Verifiable Credential
    VcJWT,
    /// JSON-LD formatted Verifiable Credential
    VcLDP,
    IanaMediaType(media_types::MediaType),
}

impl Display for DataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from(self).fmt(f)
    }
}

impl From<&DataFormat> for String {
    fn from(data_format: &DataFormat) -> Self {
        match data_format {
            DataFormat::VcJWT => "application/vc+jwt".to_string(),
            DataFormat::VcLDP => "application/vc+ldp".to_string(),
            DataFormat::IanaMediaType(media_type) => media_type.to_string(),
        }
    }
}

pub struct DescriptorBuilder {
    pub interface: Interface,
    pub method: Method,
    pub data_format: Option<DataFormat>,
}

impl DescriptorBuilder {
    pub fn build(&self) -> Result<Descriptor, Box<dyn std::error::Error>> {
        let data_format = self.data_format.as_ref().map(|f| f.to_string());

        Ok(Descriptor {
            interface: self.interface.clone(),
            method: self.method.clone(),
            data_cid: None, // TODO: Generate data_cid
            data_format,
        })
    }
}
