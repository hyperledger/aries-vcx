pub mod wrapper;

use serde::{de::IntoDeserializer, Deserialize, Deserializer, Serialize};

use did_doc::{
    did_parser::{Did, DidUrl},
    schema::{
        did_doc::{ControllerAlias, DidDocument},
        service::Service,
        utils::OneOrList,
        verification_method::{VerificationMethod, VerificationMethodKind, VerificationMethodType},
    },
};
use serde_json::Value;

use crate::{
    extra_fields::{legacy::ExtraFieldsLegacy, ExtraFieldsSov, KeyKind},
    service::{legacy::ServiceLegacy, ServiceSov},
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LegacyDidDoc {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    pub public_key: Vec<LegacyKeyAgreement>,
    #[serde(default)]
    pub authentication: Vec<LegacyAuthentication>,
    pub service: Vec<ServiceLegacy>,
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

fn legacy_key_agreement_to_verification_method(value: &LegacyKeyAgreement) -> Result<VerificationMethod, String> {
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

fn legacy_authentication_to_verification_method(
    legacy_authentication: &LegacyAuthentication,
    did: String,
    legacy_public_keys: &[LegacyKeyAgreement],
) -> Result<VerificationMethod, String> {
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

    Ok(VerificationMethod::builder(id, controller, verification_method_type)
        .add_public_key_base58(public_key_base_58)
        .build())
}

fn convert_legacy_ddo_to_new(legacy_ddo: LegacyDidDoc) -> Result<DidDocument<ExtraFieldsSov>, String> {
    let id = Did::parse(legacy_ddo.id.clone()).unwrap_or_default();
    let controller = Did::parse(legacy_ddo.id.clone()).unwrap_or_default();

    let mut builder = DidDocument::builder(id);

    for vm in &legacy_ddo.public_key {
        builder = builder.add_verification_method(legacy_key_agreement_to_verification_method(&vm)?);
    }

    for auth in &legacy_ddo.authentication {
        builder = builder.add_authentication_method(
            legacy_authentication_to_verification_method(&auth, legacy_ddo.id.clone(), &legacy_ddo.public_key).unwrap(),
        );
    }

    for service in &legacy_ddo.service {
        builder = builder.add_service(TryInto::<Service<ExtraFieldsSov>>::try_into(service.clone()).unwrap());
    }

    Ok(builder.build())
}

pub fn deserialize_legacy_or_new<'de, D>(deserializer: D) -> Result<DidDocument<ExtraFieldsSov>, D::Error>
where
    D: Deserializer<'de>,
{
    println!("deserialize_legacy_or_new");
    let val = Value::deserialize(deserializer)?;
    println!("deserialize_legacy_or_new: {:?}", val);

    match serde_json::from_value::<LegacyDidDoc>(val.clone()) {
        Ok(legacy_doc) => {
            println!("deserialize_legacy_or_new: legacy_doc");
            return Ok(convert_legacy_ddo_to_new(legacy_doc).unwrap());
        }
        Err(err) => {
            println!("deserialize_legacy_or_new: not legacy did doc: {:?}", err);
        }
    }

    serde_json::from_value::<DidDocument<ExtraFieldsSov>>(val).map_err(serde::de::Error::custom)
}
