use crate::cl::CredentialKeyCorrectnessProof;
use crate::data_types::ledger::cred_def::CredentialDefinitionId;
use crate::data_types::ledger::schema::SchemaId;
use crate::error::ValidationError;
use crate::utils::validation::Validatable;

use super::nonce::Nonce;

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialOffer {
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub key_correctness_proof: CredentialKeyCorrectnessProof,
    pub nonce: Nonce,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_name: Option<String>,
}

impl Validatable for CredentialOffer {
    fn validate(&self) -> Result<(), ValidationError> {
        self.schema_id.validate()?;
        self.cred_def_id.validate()?;
        Ok(())
    }
}
