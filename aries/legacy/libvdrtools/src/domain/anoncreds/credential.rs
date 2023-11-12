use std::collections::HashMap;

use ursa::cl::{CredentialSignature, RevocationRegistry, SignatureCorrectnessProof, Witness};

use super::{
    credential_definition::CredentialDefinitionId,
    revocation_registry_definition::RevocationRegistryId, schema::SchemaId,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Credential {
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<RevocationRegistryId>,
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
    pub const EXTRA_TAG_SUFFIX: &'static str = "_short";

    pub fn add_extra_tag_suffix(tag: &str) -> String {
        format!("{}{}", tag, Self::EXTRA_TAG_SUFFIX)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialInfo {
    pub referent: String,
    pub attrs: ShortCredentialValues,
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<RevocationRegistryId>,
    pub cred_rev_id: Option<String>,
}

pub type ShortCredentialValues = HashMap<String, String>;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialValues(pub HashMap<String, AttributeValues>);

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct AttributeValues {
    pub raw: String,
    pub encoded: String,
}
