use async_trait::async_trait;

use crate::{
    error::VcxResult, messages::connection::did::Did, messages::did_doc::service_aries::AriesService,
    xyz::primitives::revocation_registry::RevocationRegistryDefinition,
};

#[async_trait]
pub trait BaseLedger: Send + Sync {
    // multisign_request - internal
    // libindy_sign_request - internal/unused

    // returns request result as JSON
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String>;

    // returns request result as JSON
    async fn submit_request(&self, request_json: &str) -> VcxResult<String>;

    // libindy_build_schema_request - internal/testing
    // libindy_build_create_credential_def_txn - internal

    // get_txn_author_agreement - todo - used in libvcx
    // append_txn_author_agreement_to_request - internal
    // libindy_build_auth_rules_request - unused
    // libindy_build_attrib_request - internal
    // libindy_build_get_auth_rule_request - unused
    // libindy_build_get_nym_request - internal
    // libindy_build_nym_request - signus

    // returns request result as JSON
    async fn get_nym(&self, did: &str) -> VcxResult<String>;

    // returns request result as JSON
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxResult<String>;

    // get_role - internal
    // parse_response - internal

    // Schema json.
    // {
    //     id: identifier of schema
    //     attrNames: array of attribute name strings
    //     name: Schema's name string
    //     version: Schema's version string
    //     ver: Version of the Schema json
    // }
    // if submitter_did provided - use cache
    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String>;

    // libindy_build_get_cred_def_request - internal

    // if submitter_did provided, try use cache
    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxResult<String>;

    // set_endorser - todo - used in libvcx
    // endorse_transaction - todo - used in libvcx

    // build_attrib_request - internal
    // add_attr - internal
    // get_attr - internal

    async fn get_service(&self, did: &Did) -> VcxResult<AriesService>;

    // returns request result as JSON
    async fn add_service(&self, did: &str, service: &AriesService) -> VcxResult<String>;

    // libindy_build_revoc_reg_def_request - internal
    // libindy_build_revoc_reg_entry_request - internal

    // # Returns
    // Revocation Registry Definition Id and Revocation Registry Definition json.
    // {
    //     "id": string - ID of the Revocation Registry,
    //     "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
    //     "tag": string - Unique descriptive ID of the Registry,
    //     "credDefId": string - ID of the corresponding CredentialDefinition,
    //     "value": Registry-specific data {
    //         "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
    //         "maxCredNum": number - Maximum number of credentials the Registry can serve.
    //         "tailsHash": string - Hash of tails.
    //         "tailsLocation": string - Location of tails file.
    //         "publicKeys": <public_keys> - Registry's public key.
    //     },
    //     "ver": string - version of revocation registry definition json.
    // }
    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxResult<String>;

    // # Returns
    // Revocation Registry Definition Id, Revocation Registry Delta json and Timestamp.
    // {
    //     "value": Registry-specific data {
    //         prevAccum: string - previous accumulator value.
    //         accum: string - current accumulator value.
    //         issued: array<number> - an array of issued indices.
    //         revoked: array<number> an array of revoked indices.
    //     },
    //     "ver": string - version revocation registry delta json
    // }
    async fn get_rev_reg_delta_json(
        &self,
        rev_reg_id: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> VcxResult<(String, String, u64)>;

    // # Returns
    // Revocation Registry Definition Id, Revocation Registry json and Timestamp.
    // {
    //     "value": Registry-specific data {
    //         "accum": string - current accumulator value.
    //     },
    //     "ver": string - version revocation registry json
    // }
    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)>;

    // returns request result as JSON
    async fn get_ledger_txn(&self, seq_no: i32, submitter_did: Option<&str>) -> VcxResult<String>;

    // build_schema_request - todo - used in libvcx

    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxResult<()>;

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxResult<()>;

    async fn publish_rev_reg_def(
        &self,
        rev_reg_def: &RevocationRegistryDefinition,
        submitter_did: &str,
    ) -> VcxResult<()>;

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxResult<()>;
}
