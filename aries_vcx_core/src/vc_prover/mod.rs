use std::collections::HashMap;

use async_trait::async_trait;

use crate::{errors::error::VcxCoreResult,  wallet2::Wallet};

#[async_trait]
pub trait VcProver {
    type PresentationRequest;

    type SchemaId;
    type Schema;

    type CredDefId;
    type CredDef;

    type CredId;
    type Cred;

    type RevRegId;
    type RevRegDef;
    type RevStates;

    type CredReq;
    type CredReqMeta;
    type CredOffer;

    #[allow(clippy::too_many_arguments)]
    async fn create_proof(
        &self,
        wallet: &impl Wallet,
        proof_req: Self::PresentationRequest,
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: &HashMap<Self::SchemaId, Self::Schema>,
        credential_defs_json: &HashMap<Self::CredDefId, Self::CredDef>,
        revoc_states_json: Option<&HashMap<Self::RevRegId, Self::RevStates>>,
    ) -> VcxCoreResult<String>;

    async fn get_credential(
        &self,
        wallet: &impl Wallet,
        cred_id: &Self::CredId,
    ) -> VcxCoreResult<Self::Cred>;

    async fn get_credentials<W>(
        &self,
        wallet: &W,
        filter_json: Option<W::SearchFilter<'_>>,
    ) -> VcxCoreResult<String>
    // Needs a type
    where
        W: Wallet;

    async fn get_credentials_for_proof_req(
        &self,
        wallet: &impl Wallet,
        proof_request: Self::PresentationRequest,
    ) -> VcxCoreResult<String>; // Needs a type

    async fn create_credential_req(
        &self,
        wallet: &impl Wallet,
        prover_did: &str,
        cred_offer: Self::CredOffer,
        cred_def_json: Self::CredDef,
        link_secret_id: &str,
    ) -> VcxCoreResult<(Self::CredReq, Self::CredReqMeta)>;

    async fn store_credential(
        &self,
        wallet: &impl Wallet,
        cred_id: Option<Self::CredId>,
        cred_req_metadata: Self::CredReqMeta,
        cred: Self::CredReq,
        cred_def: Self::CredDef,
        rev_reg_def: Option<Self::RevRegDef>,
    ) -> VcxCoreResult<Self::CredId>;

    async fn delete_credential(
        &self,
        wallet: &impl Wallet,
        cred_id: &Self::CredId,
    ) -> VcxCoreResult<()>;

    async fn create_link_secret(
        &self,
        wallet: &impl Wallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<()>;
}
