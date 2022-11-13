use vdrtools::anoncreds;
use crate::error::{VcxError, VcxResult};

pub async fn libindy_verifier_verify_proof(
    proof_req_json: &str,
    proof_json: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    rev_reg_defs_json: &str,
    rev_regs_json: &str,
) -> VcxResult<bool> {
    anoncreds::verifier_verify_proof(
        proof_req_json,
        proof_json,
        schemas_json,
        credential_defs_json,
        rev_reg_defs_json,
        rev_regs_json,
    )
        .await
        .map_err(VcxError::from)
}



#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::indy::test_utils::{
        create_indy_proof, create_proof_with_predicate,
    };
    use crate::indy::proofs::verifier::verifier_libindy::libindy_verifier_verify_proof;
    use crate::utils::devsetup::SetupWalletPool;

    #[tokio::test]
    async fn test_prover_verify_proof() {
        let setup = SetupWalletPool::init().await;

        let (schemas, cred_defs, proof_req, proof) = create_indy_proof(setup.wallet_handle, setup.pool_handle, &setup.institution_did).await;

        let proof_validation = libindy_verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
            .await
            .unwrap();

        assert!(proof_validation);
    }

    #[tokio::test]
    async fn test_prover_verify_proof_with_predicate_success_case() {
        let setup = SetupWalletPool::init().await;

        let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(setup.wallet_handle, setup.pool_handle, &setup.institution_did, true).await;

        let proof_validation = libindy_verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
            .await
            .unwrap();

        assert!(proof_validation);
    }

    #[tokio::test]
    async fn test_prover_verify_proof_with_predicate_fail_case() {
        let setup = SetupWalletPool::init().await;

        let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(setup.wallet_handle, setup.pool_handle, &setup.institution_did, false).await;

        libindy_verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
            .await
            .unwrap_err();
    }
}
