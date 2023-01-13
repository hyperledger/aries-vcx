use std::sync::Arc;

use indy_api_types::errors::prelude::*;
use log::trace;

use crate::{
    domain::anoncreds::{
        credential_definition::{cred_defs_map_to_cred_defs_v1_map, CredentialDefinitions},
        proof::Proof,
        proof_request::ProofRequest,
        revocation_registry::{rev_regs_map_to_rev_regs_local_map, RevocationRegistries},
        revocation_registry_definition::{
            rev_reg_defs_map_to_rev_reg_defs_v1_map, RevocationRegistryDefinitions,
        },
        schema::{schemas_map_to_schemas_v1_map, Schemas},
    },
    services::VerifierService,
};

pub struct VerifierController {
    verifier_service: Arc<VerifierService>,
}

impl VerifierController {
    pub(crate) fn new(verifier_service: Arc<VerifierService>) -> VerifierController {
        VerifierController { verifier_service }
    }

    pub fn verify_proof(
        &self,
        proof_req: ProofRequest,
        proof: Proof,
        schemas: Schemas,
        cred_defs: CredentialDefinitions,
        rev_reg_defs: RevocationRegistryDefinitions,
        rev_regs: RevocationRegistries,
    ) -> IndyResult<bool> {
        trace!(
            "verify_proof > proof_req {:?} \
                proof {:?} schemas {:?} cred_defs {:?} \
                rev_reg_defs {:?} rev_regs {:?}",
            proof_req,
            proof,
            schemas,
            cred_defs,
            rev_reg_defs,
            rev_regs
        );

        let schemas = schemas_map_to_schemas_v1_map(schemas);
        let cred_defs = cred_defs_map_to_cred_defs_v1_map(cred_defs);
        let rev_reg_defs = rev_reg_defs_map_to_rev_reg_defs_v1_map(rev_reg_defs);
        let rev_regs = rev_regs_map_to_rev_regs_local_map(rev_regs);

        let valid = self.verifier_service.verify(
            &proof,
            &proof_req.value(),
            &schemas,
            &cred_defs,
            &rev_reg_defs,
            &rev_regs,
        )?;

        let res = Ok(valid);
        trace!("verify_proof < {:?}", res);
        res
    }

    pub fn generate_nonce(&self) -> IndyResult<String> {
        trace!("generate_nonce >");

        let nonce = self
            .verifier_service
            .generate_nonce()?
            .to_dec()
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize Nonce")?;

        let res = Ok(nonce);
        trace!("generate_nonce < {:?}", res);
        res
    }
}
