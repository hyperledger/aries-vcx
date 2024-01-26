use std::str::FromStr;

use crate::cl::{RevocationKeyPrivate, RevocationKeyPublic};
use crate::{error::ConversionError, impl_anoncreds_object_identifier};

use super::{cred_def::CredentialDefinitionId, issuer_id::IssuerId};

pub const CL_ACCUM: &str = "CL_ACCUM";

impl_anoncreds_object_identifier!(RevocationRegistryDefinitionId);

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum RegistryType {
    CL_ACCUM,
}

impl FromStr for RegistryType {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CL_ACCUM => Ok(Self::CL_ACCUM),
            _ => Err(ConversionError::from_msg("Invalid registry type")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub max_cred_num: u32,
    pub public_keys: RevocationRegistryDefinitionValuePublicKeys,
    pub tails_hash: String,
    pub tails_location: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValuePublicKeys {
    pub accum_key: RevocationKeyPublic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinition {
    pub issuer_id: IssuerId,
    pub revoc_def_type: RegistryType,
    pub tag: String,
    pub cred_def_id: CredentialDefinitionId,
    pub value: RevocationRegistryDefinitionValue,
}

impl Validatable for RevocationRegistryDefinition {
    fn validate(&self) -> Result<(), ValidationError> {
        self.cred_def_id.validate()?;
        self.issuer_id.validate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RevocationRegistryDefinitionPrivate {
    pub value: RevocationKeyPrivate,
}
