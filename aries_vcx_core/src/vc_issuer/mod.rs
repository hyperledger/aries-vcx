use async_trait::async_trait;

use crate::{errors::error::VcxCoreResult, wallet2::Wallet};

#[async_trait]
pub trait VcIssuer {
    type CredDefId;
    type CredDef;
    type CredDefConfig;

    type CredOffer;
    type CredReq;
    type CredValues;
    type CredRevState;
    type Cred;
    type CredRevId;

    type SigType;

    type SchemaId;
    type Schema;
    type SchemaAttrNames;

    type RevRegId;
    type RevRegDef;
    type RevRegDelta;

    async fn create_and_store_revoc_reg(
        &self,
        wallet: &impl Wallet,
        issuer_did: &str,
        cred_def_id: Self::CredDefId,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(String, String, String)>;

    async fn create_and_store_credential_def(
        &self,
        wallet: &impl Wallet,
        issuer_did: &str,
        schema_json: Self::Schema,
        tag: &str,
        signature_type: Option<Self::SigType>,
        config: Self::CredDefConfig,
    ) -> VcxCoreResult<(Self::CredDefId, Self::CredDef)>;

    async fn create_credential_offer(
        &self,
        wallet: &impl Wallet,
        cred_def_id: Self::CredDefId,
    ) -> VcxCoreResult<Self::CredOffer>;

    async fn create_credential(
        &self,
        wallet: &impl Wallet,
        cred_offer: Self::CredOffer,
        cred_req: Self::CredReq,
        cred_values: Self::CredValues,
        rev_reg_id: Option<Self::RevRegId>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(Self::Cred, Option<Self::CredRevId>)>;

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        rev_reg_def: Self::RevRegDef,
        rev_reg_delta: Self::RevRegDelta,
        timestamp: u64,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<Self::CredRevState>;

    async fn create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: Self::SchemaAttrNames,
    ) -> VcxCoreResult<(Self::SchemaId, Self::Schema)>;

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not
    // PURE Anoncreds)
    async fn revoke_credential_local(
        &self,
        wallet: &impl Wallet,
        tails_dir: &str,
        rev_reg_id: &str,
        cred_rev_id: &str,
    ) -> VcxCoreResult<()>;

    async fn get_rev_reg_delta(
        &self,
        wallet: &impl Wallet,
        rev_reg_id: &str,
    ) -> VcxCoreResult<Option<String>>;

    async fn clear_rev_reg_delta(
        &self,
        wallet: &impl Wallet,
        rev_reg_id: &str,
    ) -> VcxCoreResult<()>;
}
