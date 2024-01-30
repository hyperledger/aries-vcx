use std::str::FromStr;

use anoncreds_clsignatures::CredentialPrivateKey;

use crate::cl::{CredentialPrimaryPublicKey, CredentialPublicKey, CredentialRevocationPublicKey};
use crate::data_types::identifiers::issuer_id::IssuerId;
use crate::data_types::identifiers::schema_id::SchemaId;
use crate::error::ConversionError;
use crate::utils::validation::Validatable;

pub const CL_SIGNATURE_TYPE: &str = "CL";

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    CL,
}

impl FromStr for SignatureType {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CL_SIGNATURE_TYPE => Ok(Self::CL),
            _ => Err(ConversionError::from_msg("Invalid signature type")),
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
    pub schema_id: SchemaId,
    #[serde(rename = "type")]
    pub signature_type: SignatureType,
    pub tag: String,
    pub value: CredentialDefinitionData,
    pub issuer_id: IssuerId,
}

impl CredentialDefinition {
    pub fn get_public_key(&self) -> Result<CredentialPublicKey, ConversionError> {
        let key = CredentialPublicKey::build_from_parts(
            &self.value.primary,
            self.value.revocation.as_ref(),
        )
        .map_err(|e| e.to_string())?;
        Ok(key)
    }

    pub fn try_clone(&self) -> Result<Self, crate::Error> {
        let cred_data = CredentialDefinitionData {
            primary: self.value.primary.try_clone()?,
            revocation: self.value.revocation.clone(),
        };

        Ok(Self {
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
