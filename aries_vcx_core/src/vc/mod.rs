pub mod credx;

use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    errors::error::VcxCoreResult,
    wallet2::{Wallet, WalletRecord},
};

#[async_trait]
pub trait VcIssuer {
    type CredDefId: Send + Sync;
    type CredDef: Send + Sync;
    type CredDefPriv: Send + Sync;
    type CredKeyProof: Send + Sync;
    type CredDefConfig: Send + Sync;

    type CredOffer: Send + Sync;
    type CredReq: Send + Sync;
    type CredValues: Send + Sync;
    type Cred: Send + Sync;
    type CredRevId: Send + Sync;

    type SigType: Send + Sync;

    type SchemaId: Send + Sync;
    type Schema: Send + Sync;
    type SchemaAttrNames: Send + Sync;

    type RevRegId: Send + Sync;
    type RevRegDef: Send + Sync;
    type RevRegDefPriv: Send + Sync;
    type RevReg: Send + Sync;
    type RevRegDelta: Send + Sync;
    type RevRegInfo: Send + Sync;

    async fn create_and_store_revoc_reg<W>(
        &self,
        wallet: &W,
        issuer_did: &str,
        cred_def_id: Self::CredDefId,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(Self::RevRegId, Self::RevRegDef, Self::RevReg)>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId:
            OtherFrom<Self::CredDefId> + OtherFrom<Self::RevRegId> + Send + Sync,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevReg: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>;

    async fn create_and_store_credential_def<W>(
        &self,
        wallet: &W,
        issuer_did: &str,
        schema: Self::Schema,
        tag: &str,
        signature_type: Option<Self::SigType>,
        config: Self::CredDefConfig,
    ) -> VcxCoreResult<(Self::CredDefId, Self::CredDef)>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId:
            OtherFrom<Self::CredDefId> + OtherFrom<Self::SchemaId> + Send + Sync,
        Self::Schema: WalletRecord<W, RecordId = W::RecordId>,
        Self::SchemaId: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredKeyProof: WalletRecord<W, RecordId = W::RecordId>;

    async fn create_credential_offer<W>(
        &self,
        wallet: &W,
        cred_def_id: Self::CredDefId,
    ) -> VcxCoreResult<Self::CredOffer>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId: OtherFrom<Self::CredDefId> + Send + Sync,
        Self::SchemaId: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredKeyProof: WalletRecord<W, RecordId = W::RecordId>;

    async fn create_credential<W>(
        &self,
        wallet: &W,
        cred_offer: Self::CredOffer,
        cred_req: Self::CredReq,
        cred_values: Self::CredValues,
        rev_reg_id: Option<Self::RevRegId>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(Self::Cred, Option<Self::CredRevId>)>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId:
            OtherFrom<Self::CredDefId> + OtherFrom<Self::RevRegId> + Send + Sync,
        Self::Schema: WalletRecord<W, RecordId = W::RecordId>,
        Self::SchemaId: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevReg: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredKeyProof: WalletRecord<W, RecordId = W::RecordId>;

    async fn create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: Self::SchemaAttrNames,
    ) -> VcxCoreResult<(Self::SchemaId, Self::Schema)>;

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not
    // PURE Anoncreds)
    async fn revoke_credential<W>(
        &self,
        wallet: &W,
        tails_dir: &str,
        rev_reg_id: Self::RevRegId,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId:
            OtherFrom<Self::CredDefId> + OtherFrom<Self::RevRegId> + Send + Sync,
        Self::RevRegDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevReg: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDelta: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>;

    async fn get_revocation_delta<W>(
        &self,
        wallet: &W,
        rev_reg_id: Self::RevRegId,
    ) -> VcxCoreResult<Option<Self::RevRegDelta>>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId:
            OtherFrom<Self::CredDefId> + OtherFrom<Self::RevRegId> + Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordId = W::RecordId>;

    async fn clear_revocation_delta<W>(
        &self,
        wallet: &W,
        rev_reg_id: Self::RevRegId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId:
            OtherFrom<Self::CredDefId> + OtherFrom<Self::RevRegId> + Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordId = W::RecordId>;
}

#[async_trait]
pub trait VcProver {
    type PresentationRequest;

    type SchemaId;
    type Schema;

    type CredDefId;
    type CredDef;

    type CredId;
    type Cred;
    type CredRevId: Send + Sync;
    type CredRevState: Send + Sync;
    type CredRevStateParts: Send + Sync;

    type RevRegId;
    type RevRegDef;
    type RevStates;

    type CredReq;
    type CredReqMeta;
    type CredOffer;

    type LinkSecretId;

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

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        cred_rev_state_parts: Self::CredRevStateParts,
        timestamp: u64,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<Self::CredRevState>;

    async fn create_credential_req(
        &self,
        wallet: &impl Wallet,
        prover_did: &str,
        cred_offer: Self::CredOffer,
        cred_def_json: Self::CredDef,
        link_secret_id: Self::LinkSecretId,
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
        link_secret_id: Self::LinkSecretId,
    ) -> VcxCoreResult<()>;
}

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

/// Because indy-credx doesn't find it useful to implement From
pub trait OtherFrom<T> {
    fn other_from(value: T) -> Self;
}
