use crate::{data_types::ledger::cred_def::SignatureType, utils::validation::Validatable};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CredentialDefinitionConfig {
    pub support_revocation: bool,
    pub tag: String,
    pub signature_type: SignatureType,
}

impl Validatable for CredentialDefinitionConfig {}
