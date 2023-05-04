use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;

/// Trait defining standard 'ledger' related functionality.
#[async_trait]
pub trait LedgerRead {
    type Schema;

    type CredDef;

    type RevRegDef;
    type RevRegDelta;
    type RevReg;

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
    async fn get_schema(&self, schema_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<Self::Schema>;

    // if submitter_did provided, try use cache
    // TO CONSIDER - do we need to return the cred def ID in a tuple? is it ever different to the input?
    async fn get_cred_def(&self, cred_def_id: &str, submitter_did: Option<&str>) -> VcxCoreResult<Self::CredDef>;

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
    async fn get_rev_reg_def_json(&self, rev_reg_id: &str) -> VcxCoreResult<Self::RevRegDef>;

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
    ) -> VcxCoreResult<(String, Self::RevRegDelta, u64)>;

    // # Returns
    // Revocation Registry Definition Id, Revocation Registry json and Timestamp.
    // {
    //     "value": Registry-specific data {
    //         "accum": string - current accumulator value.
    //     },
    //     "ver": string - version revocation registry json
    // }
    async fn get_rev_reg(&self, rev_reg_id: &str, timestamp: u64) -> VcxCoreResult<(String, Self::RevReg, u64)>;
}

#[async_trait]
pub trait LedgerWrite: LedgerRead {
    async fn publish_schema(
        &self,
        schema_json: Self::Schema,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()>;

    async fn publish_cred_def(&self, cred_def_json: Self::CredDef, submitter_did: &str) -> VcxCoreResult<()>;

    async fn publish_rev_reg_def(&self, rev_reg_def: Self::RevRegDef, submitter_did: &str) -> VcxCoreResult<()>;

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()>;

    async fn publish_rev_reg(&self, rev_reg_id: &str, rev_reg: Self::RevReg, timestamp: u64) -> VcxCoreResult<()>;
}
