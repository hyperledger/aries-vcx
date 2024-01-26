use std::collections::HashMap;

use crate::cl::Proof;
use crate::error::ValidationError;
use crate::utils::validation::Validatable;

use super::{
    cred_def::CredentialDefinitionId, rev_reg_def::RevocationRegistryDefinitionId, schema::SchemaId,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Presentation {
    pub proof: Proof,
    pub requested_proof: RequestedProof,
    pub identifiers: Vec<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct RequestedProof {
    pub revealed_attrs: HashMap<String, RevealedAttributeInfo>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub revealed_attr_groups: HashMap<String, RevealedAttributeGroupInfo>,
    #[serde(default)]
    pub self_attested_attrs: HashMap<String, String>,
    #[serde(default)]
    pub unrevealed_attrs: HashMap<String, SubProofReferent>,
    #[serde(default)]
    pub predicates: HashMap<String, SubProofReferent>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct SubProofReferent {
    pub sub_proof_index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct RevealedAttributeInfo {
    pub sub_proof_index: u32,
    pub raw: String,
    pub encoded: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RevealedAttributeGroupInfo {
    pub sub_proof_index: u32,
    pub values: HashMap<String /* attribute name */, AttributeValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct AttributeValue {
    pub raw: String,
    pub encoded: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Identifier {
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<RevocationRegistryDefinitionId>,
    pub timestamp: Option<u64>,
}

impl Validatable for Presentation {
    fn validate(&self) -> Result<(), ValidationError> {
        for identifier in &self.identifiers {
            identifier.schema_id.validate()?;
            identifier.cred_def_id.validate()?;
            identifier
                .rev_reg_id
                .as_ref()
                .map(Validatable::validate)
                .transpose()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_requested_proof_with_empty_revealed_attr_groups() {
        let mut req_proof_old: RequestedProof = Default::default();
        req_proof_old.revealed_attrs.insert(
            "attr1".to_string(),
            RevealedAttributeInfo {
                sub_proof_index: 0,
                raw: "123".to_string(),
                encoded: "123".to_string(),
            },
        );
        let json = json!(req_proof_old).to_string();
        let req_proof: RequestedProof = serde_json::from_str(&json).unwrap();
        assert!(req_proof.revealed_attr_groups.is_empty())
    }
}
