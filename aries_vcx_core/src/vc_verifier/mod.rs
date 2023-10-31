use std::collections::HashMap;

use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait VcVerifier {
    type PresentationRequest;
    type Presentation;

    type SchemaId;
    type Schema;

    type CredDefId;
    type CredDef;

    type RevRegId;
    type RevRegDef;
    type RevStates;

    async fn verify_proof(
        &self,
        proof_request: Self::PresentationRequest,
        proof: Self::Presentation,
        schemas: &HashMap<Self::SchemaId, Self::Schema>,
        credential_defs: &HashMap<Self::CredDefId, Self::CredDef>,
        rev_reg_defs: Option<&HashMap<Self::RevRegId, Self::RevRegDef>>,
        rev_regs: Option<&HashMap<Self::RevRegId, Self::RevStates>>,
    ) -> VcxCoreResult<bool>;

    async fn generate_nonce(&self) -> VcxCoreResult<String>;
}
