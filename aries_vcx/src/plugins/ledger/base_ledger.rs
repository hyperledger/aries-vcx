use async_trait::async_trait;

use crate::{common::primitives::revocation_registry::RevocationRegistryDefinition, errors::error::VcxResult};

/// Trait defining standard 'ledger' related functionality.
#[async_trait]
pub trait BaseLedger: Send + Sync {
    // returns request result as JSON
    async fn sign_and_submit_request(&self, submitter_did: &str, request_json: &str) -> VcxResult<String>;

    // returns request result as JSON
    async fn submit_request(&self, request_json: &str) -> VcxResult<String>;

    // endorsers/multi signs a request, submits to ledger, and verifies successful result
    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxResult<()>;

    // adds endorser to request and signs with submitter_did, returns the transaction ready for endorser
    // to take
    async fn set_endorser(&self, submitter_did: &str, request: &str, endorser: &str) -> VcxResult<String>;

    async fn get_txn_author_agreement(&self) -> VcxResult<String>;

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

    // Schema json.
    // {
    //     id: identifier of schema
    //     attrNames: array of attribute name strings
    //     name: Schema's name string
    //     version: Schema's version string
    //     ver: Version of the Schema json
    // }
    // if submitter_did provided - use cache
    // TO CONSIDER - do we need to return the schema ID in a tuple? is it ever different to the input?
    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxResult<String>;

    // if submitter_did provided, try use cache
    // TO CONSIDER - do we need to return the cred def ID in a tuple? is it ever different to the input?
    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxResult<String>;

    // returns request result as JSON
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> VcxResult<String>;

    // returns request result as JSON
    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxResult<String>;

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
    // TO CONSIDER - do we need to return the rev reg id in a tuple? is it ever different to the input?
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

    // returns request as JSON
    async fn build_schema_request(&self, submitter_did: &str, schema_json: &str) -> VcxResult<String>;

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
