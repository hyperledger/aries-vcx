use std::str::FromStr;

use anoncreds_clsignatures::CredentialPrivateKey;

use crate::{
    cl::{CredentialPrimaryPublicKey, CredentialPublicKey, CredentialRevocationPublicKey},
    data_types::identifiers::{
        cred_def_id::CredentialDefinitionId, issuer_id::IssuerId, schema_id::SchemaId,
    },
    utils::validation::Validatable,
};

pub const CL_SIGNATURE_TYPE: &str = "CL";

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SignatureType {
    #[default]
    CL,
}

impl FromStr for SignatureType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CL_SIGNATURE_TYPE => Ok(Self::CL),
            _ => Err(crate::Error::from_msg(
                crate::ErrorKind::ConversionError,
                "Invalid signature type",
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinitionData {
    pub primary: CredentialPrimaryPublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation: Option<CredentialRevocationPublicKey>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDefinition {
    pub id: CredentialDefinitionId,
    pub schema_id: SchemaId,
    #[serde(rename = "type")]
    pub signature_type: SignatureType,
    pub tag: String,
    pub value: CredentialDefinitionData,
    pub issuer_id: IssuerId,
}

impl CredentialDefinition {
    pub fn get_public_key(&self) -> Result<CredentialPublicKey, crate::Error> {
        CredentialPublicKey::build_from_parts(&self.value.primary, self.value.revocation.as_ref())
            .map_err(|e| crate::Error::from_msg(crate::ErrorKind::ConversionError, e.to_string()))
    }

    pub fn try_clone(&self) -> Result<Self, crate::Error> {
        let cred_data = CredentialDefinitionData {
            primary: self.value.primary.try_clone()?,
            revocation: self.value.revocation.clone(),
        };

        Ok(Self {
            id: self.id.clone(),
            schema_id: self.schema_id.clone(),
            signature_type: self.signature_type,
            tag: self.tag.clone(),
            value: cred_data,
            issuer_id: self.issuer_id.clone(),
        })
    }
}

impl Validatable for CredentialDefinition {
    fn validate(&self) -> Result<(), crate::error::Error> {
        self.schema_id.validate()?;
        self.issuer_id.validate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialDefinitionPrivate {
    pub value: CredentialPrivateKey,
}
