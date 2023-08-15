pub mod wrapper;

use serde::{Deserialize, Serialize};

use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::ControllerAlias,
        service::Service,
        utils::OneOrList,
        verification_method::{VerificationMethod, VerificationMethodKind, VerificationMethodType},
    },
};

use crate::extra_fields::{legacy::ExtraFieldsLegacy, KeyKind};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyDidDoc {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    pub public_key: Vec<LegacyKeyAgreement>,
    #[serde(default)]
    pub authentication: Vec<LegacyAuthentication>,
    pub service: Vec<Service<ExtraFieldsLegacy>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyKeyAgreement {
    pub id: String,
    #[serde(rename = "type")]
    pub verification_method_type: String,
    pub controller: String,
    #[serde(rename = "publicKeyBase58")]
    pub public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyAuthentication {
    #[serde(rename = "type")]
    pub verification_method_type: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

impl LegacyDidDoc {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn service(&self) -> &[Service<ExtraFieldsLegacy>] {
        &self.service
    }

    pub fn controller(&self) -> Option<ControllerAlias> {
        self.public_key
            .first()
            .map(|pk| OneOrList::One(pk.controller.parse().unwrap_or_default()))
    }

    pub fn verification_method(&self) -> Vec<VerificationMethod> {
        self.public_key.iter().map(|pk| pk.try_into().unwrap()).collect()
    }

    pub fn authentication(&self) -> Vec<VerificationMethodKind> {
        self.authentication
            .iter()
            .map(|pk| legacy_authentication_to_verification_method(pk, self.id.clone(), &self.public_key).unwrap())
            .collect()
    }
}

impl TryFrom<&LegacyKeyAgreement> for VerificationMethod {
    type Error = String;

    fn try_from(value: &LegacyKeyAgreement) -> Result<Self, Self::Error> {
        let LegacyKeyAgreement {
            id,
            verification_method_type: _,
            controller,
            public_key_base_58,
        } = value;
        let id = DidUrl::parse(id.clone()).unwrap_or_default();
        let controller = Did::parse(controller.clone()).unwrap_or_default();
        let verification_method_type = VerificationMethodType::X25519KeyAgreementKey2019;

        Ok(VerificationMethod::builder(id, controller, verification_method_type)
            .add_public_key_base58(public_key_base_58.clone())
            .build())
    }
}

fn legacy_authentication_to_verification_method(
    legacy_authentication: &LegacyAuthentication,
    did: String,
    legacy_public_keys: &[LegacyKeyAgreement],
) -> Result<VerificationMethodKind, String> {
    let id = DidUrl::parse(did.clone()).unwrap_or_default();
    let controller = Did::parse(did).unwrap_or_default();
    let verification_method_type = VerificationMethodType::Ed25519VerificationKey2018;

    let fragment = legacy_authentication.public_key.split('#').last().unwrap();
    let public_key_base_58 = legacy_public_keys
        .iter()
        .find(|pk| pk.id.ends_with(fragment))
        .ok_or_else(|| format!("Public key with id {} not found", fragment))?
        .public_key_base_58
        .clone();

    Ok(VerificationMethodKind::Resolved(
        VerificationMethod::builder(id, controller, verification_method_type)
            .add_public_key_base58(public_key_base_58)
            .build(),
    ))
}
