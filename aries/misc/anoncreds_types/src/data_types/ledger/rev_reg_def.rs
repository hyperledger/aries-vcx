use std::str::FromStr;

use anoncreds_clsignatures::RevocationKeyPrivate;

use crate::{
    cl::RevocationKeyPublic,
    data_types::identifiers::{
        cred_def_id::CredentialDefinitionId, issuer_id::IssuerId,
        rev_reg_def_id::RevocationRegistryDefinitionId,
    },
    utils::validation::Validatable,
};

pub const CL_ACCUM: &str = "CL_ACCUM";

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum RegistryType {
    CL_ACCUM,
}

impl FromStr for RegistryType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CL_ACCUM => Ok(Self::CL_ACCUM),
            _ => Err(crate::Error::from_msg(
                crate::ErrorKind::ConversionError,
                "Invalid registry type",
            )),
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
    pub id: RevocationRegistryDefinitionId,
    pub issuer_id: IssuerId,
    pub revoc_def_type: RegistryType,
    pub tag: String,
    pub cred_def_id: CredentialDefinitionId,
    pub value: RevocationRegistryDefinitionValue,
}

impl Validatable for RevocationRegistryDefinition {
    fn validate(&self) -> Result<(), crate::error::Error> {
        self.cred_def_id.validate()?;
        self.issuer_id.validate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RevocationRegistryDefinitionPrivate {
    pub value: RevocationKeyPrivate,
}
