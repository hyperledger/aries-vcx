use super::nonce::Nonce;
use crate::{
    cl::CredentialKeyCorrectnessProof,
    data_types::identifiers::{cred_def_id::CredentialDefinitionId, schema_id::SchemaId},
    utils::validation::Validatable,
};

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
