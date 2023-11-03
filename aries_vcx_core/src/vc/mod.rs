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
        cred_def_id: &Self::CredDefId,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(Self::RevRegId, Self::RevRegDef, Self::RevReg)>
    where
        W: Wallet + Send + Sync,
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::CredDef: WalletRecord<W>,
        for<'b> Self::RevReg: WalletRecord<W, RecordIdRef<'b> = &'b Self::RevRegId>,
        for<'b> Self::RevRegDef: WalletRecord<W, RecordIdRef<'b> = &'b Self::RevRegId>,
        for<'b> Self::RevRegDefPriv: WalletRecord<W, RecordIdRef<'b> = &'b Self::RevRegId>,
        for<'b> Self::RevRegInfo: WalletRecord<W, RecordIdRef<'b> = &'b Self::RevRegId>;

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
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        for<'a> Self::Schema: WalletRecord<W, RecordIdRef<'a> = &'a Self::SchemaId>,
        for<'a> Self::SchemaId: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        for<'a> Self::CredDef: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        for<'a> Self::CredDefPriv: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        for<'a> Self::CredKeyProof: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>;

    async fn create_credential_offer<'a, W>(
        &self,
        wallet: &W,
        cred_def_id: &'a Self::CredDefId,
    ) -> VcxCoreResult<Self::CredOffer>
    where
        W: Wallet + Send + Sync,
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::SchemaId: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        Self::CredDef: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        Self::CredKeyProof: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>;

    async fn create_credential<'a, W>(
        &self,
        wallet: &W,
        cred_offer: Self::CredOffer,
        cred_req: Self::CredReq,
        cred_values: Self::CredValues,
        rev_reg_id: Option<&'a Self::RevRegId>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(Self::Cred, Option<Self::CredRevId>)>
    where
        W: Wallet + Send + Sync,
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        for<'b> Self::Schema: WalletRecord<W, RecordIdRef<'b> = &'b Self::SchemaId>,
        for<'b> Self::SchemaId: WalletRecord<W, RecordIdRef<'b> = &'b Self::CredDefId>,
        for<'b> Self::CredDef: WalletRecord<W, RecordIdRef<'b> = &'b Self::CredDefId>,
        for<'b> Self::CredDefPriv: WalletRecord<W, RecordIdRef<'b> = &'b Self::CredDefId>,
        for<'b> Self::CredKeyProof: WalletRecord<W, RecordIdRef<'b> = &'b Self::CredDefId>,
        Self::RevRegDef: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevReg: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevRegInfo: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>;

    async fn create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: Self::SchemaAttrNames,
    ) -> VcxCoreResult<(Self::SchemaId, Self::Schema)>;

    async fn revoke_credential<'a, W>(
        &self,
        wallet: &W,
        rev_reg_id: &'a Self::RevRegId,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::RevReg: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevRegDef: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevRegInfo: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        Self::RevRegDelta: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
        for<'b> Self::CredDef: WalletRecord<W, RecordIdRef<'b> = &'b Self::CredDefId>;

    async fn get_revocation_delta<'a, W>(
        &self,
        wallet: &W,
        rev_reg_id: &'a Self::RevRegId,
    ) -> VcxCoreResult<Option<Self::RevRegDelta>>
    where
        W: Wallet + Send + Sync,
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>;

    async fn clear_revocation_delta<'a, W>(
        &self,
        wallet: &W,
        rev_reg_id: &'a Self::RevRegId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>;
}

#[async_trait]
pub trait VcProver {
    type Presentation: Send + Sync;
    type PresentationRequest: Send + Sync;

    type SchemaId: Send + Sync;
    type Schema: Send + Sync;

    type CredDefId: Send + Sync;
    type CredDef: Send + Sync;

    type CredId: Send + Sync;
    type Cred: Send + Sync;
    type CredRevId: Send + Sync;
    type CredRevState: Send + Sync;
    // Enables specifying things like `(RevocationRegistry, RevocationRegistryDelta)`
    // as well something else meant to work with different credentials.
    type CredRevStateParts: Send + Sync;

    type RevRegId: Send + Sync;
    type RevRegDef: Send + Sync;
    type RevStates: Send + Sync;

    type CredReq: Send + Sync;
    type CredReqMeta: Send + Sync;
    type CredOffer: Send + Sync;

    type LinkSecretId: Send + Sync;
    type LinkSecret: Send + Sync;

    #[allow(clippy::too_many_arguments)]
    async fn create_presentation<W>(
        &self,
        wallet: &W,
        pres_req: Self::PresentationRequest,
        requested_credentials: &str, // needs a type
        link_secret_id: &Self::LinkSecretId,
        schemas: &HashMap<Self::SchemaId, Self::Schema>,
        cred_defs: &HashMap<Self::CredDefId, Self::CredDef>,
        rev_states: Option<&HashMap<Self::RevRegId, Self::RevStates>>,
    ) -> VcxCoreResult<Self::Presentation>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::CredId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::LinkSecretId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::Cred: WalletRecord<W>,
        Self::LinkSecret: WalletRecord<W>;

    async fn get_credentials_for_proof_req<W>(
        &self,
        wallet: &W,
        proof_request: Self::PresentationRequest,
    ) -> VcxCoreResult<String>
    // Needs a type
    where
        // Bound too limiting
        for<'a> W: Wallet<SearchFilter<'a> = &'a str> + Send + Sync,
        Self::Cred: WalletRecord<W, RecordId = String>;

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        cred_rev_state_parts: Self::CredRevStateParts,
        timestamp: u64,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<Self::CredRevState>;

    async fn create_credential_req<W>(
        &self,
        wallet: &W,
        prover_did: &str,
        cred_offer: &Self::CredOffer,
        cred_def: &Self::CredDef,
        link_secret_id: &Self::LinkSecretId,
    ) -> VcxCoreResult<(Self::CredReq, Self::CredReqMeta)>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::LinkSecretId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::LinkSecret: WalletRecord<W>;

    async fn store_credential<W>(
        &self,
        wallet: &W,
        cred_id: Option<Self::CredId>,
        cred_req_metadata: &Self::CredReqMeta,
        cred: &mut Self::Cred,
        cred_def: &Self::CredDef,
        rev_reg_def: Option<&Self::RevRegDef>,
    ) -> VcxCoreResult<Self::CredId>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::LinkSecretId: AsRef<<W as Wallet>::RecordIdRef>,
        Self::LinkSecret: WalletRecord<W>,
        for<'a> Self::Cred: WalletRecord<W, RecordIdRef<'a> = Self::CredId>;

    async fn create_link_secret<W>(
        &self,
        wallet: &W,
        link_secret_id: Self::LinkSecretId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::LinkSecretId: AsRef<<W as Wallet>::RecordIdRef>,
        for<'a> Self::LinkSecret: WalletRecord<W, RecordIdRef<'a> = Self::LinkSecretId>;
}

#[async_trait]
pub trait VcVerifier {
    type PresentationRequest: Send + Sync;
    type Presentation: Send + Sync;

    type SchemaId: Send + Sync;
    type Schema: Send + Sync;

    type CredDefId: Send + Sync;
    type CredDef: Send + Sync;

    type RevRegId: Send + Sync;
    type RevRegDef: Send + Sync;
    type RevStates: Send + Sync;

    async fn verify_proof(
        &self,
        pres_request: &Self::PresentationRequest,
        presentation: &Self::Presentation,
        schemas: &HashMap<Self::SchemaId, Self::Schema>,
        credential_defs: &HashMap<Self::CredDefId, Self::CredDef>,
        rev_reg_defs: Option<&HashMap<Self::RevRegId, Self::RevRegDef>>,
        rev_regs: Option<&HashMap<Self::RevRegId, Self::RevStates>>,
    ) -> VcxCoreResult<bool>;

    async fn generate_nonce(&self) -> VcxCoreResult<String>;
}
