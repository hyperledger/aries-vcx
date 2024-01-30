use crate::cl::CredentialKeyCorrectnessProof;
use crate::data_types::identifiers::cred_def_id::CredentialDefinitionId;
use crate::data_types::identifiers::schema_id::SchemaId;
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
    fn validate(&self) -> Result<(), crate::error::Error> {
        self.schema_id.validate()?;
        self.cred_def_id.validate()?;
        Ok(())
    }
}
