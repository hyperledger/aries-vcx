pub mod wrapper;

use serde::{Deserialize, Serialize};

use did_doc::schema::service::Service;

use crate::extra_fields::legacy::ExtraFieldsLegacy;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyDidDoc {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    pub public_key: Vec<LegacyVerificationMethod>,
    #[serde(default)]
    pub authentication: Vec<LegacyAuthentication>,
    pub service: Vec<Service<ExtraFieldsLegacy>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyVerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub controller: String,
    #[serde(rename = "publicKeyBase58")]
    pub public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyAuthentication {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

impl LegacyDidDoc {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn authentication(&self) -> &[LegacyAuthentication] {
        &self.authentication
    }

    pub fn service(&self) -> &[Service<ExtraFieldsLegacy>] {
        &self.service
    }
}
