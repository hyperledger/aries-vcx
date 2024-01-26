use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

use crate::cl::{CredentialSignature, RevocationRegistry, SignatureCorrectnessProof, Witness};
use crate::error::{ConversionError, ValidationError};
use crate::utils::validation::Validatable;

use super::rev_reg_def::RevocationRegistryDefinitionId;
use super::{cred_def::CredentialDefinitionId, schema::SchemaId};

#[derive(Debug, Deserialize, Serialize)]
pub struct Credential {
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<RevocationRegistryDefinitionId>,
    pub values: CredentialValues,
    pub signature: CredentialSignature,
    pub signature_correctness_proof: SignatureCorrectnessProof,
    pub rev_reg: Option<RevocationRegistry>,
    pub witness: Option<Witness>,
}

impl Credential {
    pub const QUALIFIABLE_TAGS: [&'static str; 5] = [
        "issuer_did",
        "cred_def_id",
        "schema_id",
        "schema_issuer_did",
        "rev_reg_id",
    ];

    pub fn try_clone(&self) -> Result<Self, ConversionError> {
        Ok(Self {
            schema_id: self.schema_id.clone(),
            cred_def_id: self.cred_def_id.clone(),
            rev_reg_id: self.rev_reg_id.clone(),
            values: self.values.clone(),
            signature: self.signature.try_clone().map_err(|e| e.to_string())?,
            signature_correctness_proof: self
                .signature_correctness_proof
                .try_clone()
                .map_err(|e| e.to_string())?,
            rev_reg: self.rev_reg.clone(),
            witness: self.witness.clone(),
        })
    }
}

impl Validatable for Credential {
    fn validate(&self) -> Result<(), ValidationError> {
        self.values.validate()?;
        self.schema_id.validate()?;
        self.cred_def_id.validate()?;
        self.rev_reg_id
            .as_ref()
            .map(Validatable::validate)
            .transpose()?;

        if self.rev_reg_id.is_some() && (self.witness.is_none() || self.rev_reg.is_none()) {
            return Err("Credential validation failed: `witness` and `rev_reg` must be passed for revocable Credential".into());
        }

        if self.values.0.is_empty() {
            return Err("Credential validation failed: `values` is empty".into());
        }

        Ok(())
    }
}

// #[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
// pub struct RawCredentialValues(pub HashMap<String, String>);
//
// #[cfg(feature = "zeroize")]
// impl Drop for RawCredentialValues {
//     fn drop(&mut self) {
//         self.zeroize();
//     }
// }
//
// #[cfg(feature = "zeroize")]
// impl Zeroize for RawCredentialValues {
//     fn zeroize(&mut self) {
//         for attr in self.0.values_mut() {
//             attr.zeroize();
//         }
//     }
// }
//
// impl Validatable for RawCredentialValues {
//     fn validate(&self) -> Result<(), ValidationError> {
//         if self.0.is_empty() {
//             return Err("RawCredentialValues validation failed: empty list has been passed".into());
//         }
//
//         Ok(())
//     }
// }
//
// impl From<&CredentialValues> for RawCredentialValues {
//     fn from(values: &CredentialValues) -> Self {
//         RawCredentialValues(
//             values
//                 .0
//                 .iter()
//                 .map(|(attribute, values)| (attribute.to_owned(), values.raw.to_owned()))
//                 .collect(),
//         )
//     }
// }

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct CredentialValues(pub HashMap<String, AttributeValues>);

#[cfg(feature = "zeroize")]
impl Drop for CredentialValues {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl Validatable for CredentialValues {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.0.is_empty() {
            return Err("CredentialValues validation failed: empty list has been passed".into());
        }

        Ok(())
    }
}

#[cfg(feature = "zeroize")]
impl Zeroize for CredentialValues {
    fn zeroize(&mut self) {
        for attr in self.0.values_mut() {
            attr.zeroize();
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "zeroize", derive(Zeroize))]
pub struct AttributeValues {
    pub raw: String,
    pub encoded: String,
}
