use std::collections::HashSet;

use async_trait::async_trait;
use indy_credx::{
    issuer,
    tails::TailsFileWriter,
    types::{
        AttributeNames, Credential, CredentialDefinition, CredentialDefinitionConfig,
        CredentialDefinitionId, CredentialDefinitionPrivate, CredentialKeyCorrectnessProof,
        CredentialOffer, CredentialRequest, CredentialRevocationState, CredentialValues,
        IssuanceType, RegistryType, RevocationRegistry, RevocationRegistryDefinition,
        RevocationRegistryDefinitionPrivate, RevocationRegistryDelta, RevocationRegistryId, Schema,
        SchemaId, SignatureType,
    },
};
use serde::{Deserialize, Serialize};

use super::{OtherFrom, VcIssuer};
use crate::{
    errors::error::VcxCoreResult,
    wallet2::{Wallet, WalletRecord},
};

pub struct IndyCredxIssuer;

#[async_trait]
impl VcIssuer for IndyCredxIssuer {
    type CredDefId = CredentialDefinitionId;
    type CredDef = CredentialDefinition;
    type CredDefPriv = CredentialDefinitionPrivate;
    type CredKeyProof = CredentialKeyCorrectnessProof;
    type CredDefConfig = CredentialDefinitionConfig;

    type CredOffer = CredentialOffer;
    type CredReq = CredentialRequest;
    type CredValues = CredentialValues;
    type CredRevState = CredentialRevocationState;
    type CredRevStateParts = (Self::RevRegDef, Self::RevRegDelta);
    type Cred = Credential;
    type CredRevId = u32;

    type SigType = SignatureType;

    type SchemaId = SchemaId;
    type Schema = Schema;
    type SchemaAttrNames = AttributeNames;

    type RevRegId = RevocationRegistryId;
    type RevReg = RevocationRegistry;
    type RevRegDef = RevocationRegistryDefinition;
    type RevRegDefPriv = RevocationRegistryDefinitionPrivate;
    type RevRegDelta = RevocationRegistryDelta;
    type RevRegInfo = RevocationRegistryInfo;

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
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>,
    {
        let issuer_did = issuer_did.to_owned().into();

        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_owned()));

        let cred_def = wallet.get(&W::RecordId::other_from(cred_def_id)).await?;

        let rev_reg_id = issuer::make_revocation_registry_id(
            &issuer_did,
            &cred_def,
            tag,
            RegistryType::CL_ACCUM,
        )?;

        let res_rev_reg = wallet.get(&W::RecordId::other_from(rev_reg_id)).await;
        let res_rev_reg_def = wallet.get(&W::RecordId::other_from(rev_reg_id)).await;

        if let (Ok(rev_reg), Ok(rev_reg_def)) = (res_rev_reg, res_rev_reg_def) {
            return Ok((rev_reg_id, rev_reg, rev_reg_def));
        }

        let (rev_reg_def, rev_reg_def_priv, rev_reg, _rev_reg_delta) =
            issuer::create_revocation_registry(
                &issuer_did,
                &cred_def,
                tag,
                RegistryType::CL_ACCUM,
                IssuanceType::ISSUANCE_BY_DEFAULT,
                max_creds,
                &mut tails_writer,
            )?;

        // Store stuff in wallet
        let rev_reg_info = RevocationRegistryInfo {
            id: rev_reg_id.clone(),
            curr_id: 0,
            used_ids: HashSet::new(),
        };

        wallet
            .add(rev_reg_info.into_wallet_record(W::RecordId::other_from(rev_reg_id))?)
            .await?;
        wallet
            .add(rev_reg_def.into_wallet_record(W::RecordId::other_from(rev_reg_id))?)
            .await?;
        wallet
            .add(rev_reg_def_priv.into_wallet_record(W::RecordId::other_from(rev_reg_id))?)
            .await?;
        wallet
            .add(rev_reg.into_wallet_record(W::RecordId::other_from(rev_reg_id))?)
            .await?;

        Ok((rev_reg_id, rev_reg_def, rev_reg))
    }

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
        Self::CredKeyProof: WalletRecord<W, RecordId = W::RecordId>,
    {
        let issuer_did = issuer_did.to_owned().into();

        let sig_type = signature_type.unwrap_or(SignatureType::CL);

        let schema_seq_no = match &schema {
            Schema::SchemaV1(s) => s.seq_no,
        };

        let cred_def_id = issuer::make_credential_definition_id(
            &issuer_did,
            schema.id(),
            schema_seq_no,
            tag,
            sig_type,
        )?;

        // If cred def already exists, return it
        if let Ok(cred_def) = wallet.get(&W::RecordId::other_from(cred_def_id)).await {
            return Ok((cred_def_id, cred_def));
        }

        // Otherwise, create cred def
        let (cred_def, cred_def_priv, cred_key_correctness_proof) =
            issuer::create_credential_definition(&issuer_did, &schema, tag, sig_type, config)?;

        wallet
            .add(cred_def.into_wallet_record(W::RecordId::other_from(cred_def_id))?)
            .await?;

        wallet
            .add(cred_def_priv.into_wallet_record(W::RecordId::other_from(cred_def_id))?)
            .await?;

        wallet
            .add(
                cred_key_correctness_proof
                    .into_wallet_record(W::RecordId::other_from(cred_def_id))?,
            )
            .await?;

        let schema_id = SchemaId(schema.id().0);
        wallet
            .add(schema.into_wallet_record(W::RecordId::other_from(schema_id))?)
            .await?;

        wallet
            .add(schema_id.into_wallet_record(W::RecordId::other_from(cred_def_id))?)
            .await?;

        // Return the ID and the cred def
        Ok((cred_def_id, cred_def))
    }

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
        cred_rev_state_parts: Self::CredRevStateParts,
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
    async fn revoke_credential(
        &self,
        wallet: &impl Wallet,
        tails_dir: &str,
        rev_reg_id: Self::RevRegId,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<()>;

    async fn get_revocation_delta(
        &self,
        wallet: &impl Wallet,
        rev_reg_id: Self::RevRegId,
    ) -> VcxCoreResult<Option<Self::RevRegDelta>>;

    async fn clear_revocation_delta(
        &self,
        wallet: &impl Wallet,
        rev_reg_id: Self::RevRegId,
    ) -> VcxCoreResult<()>;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}

impl OtherFrom<CredentialDefinitionId> for String {
    fn other_from(value: CredentialDefinitionId) -> Self {
        value.0
    }
}

impl OtherFrom<RevocationRegistryId> for String {
    fn other_from(value: RevocationRegistryId) -> Self {
        value.0
    }
}

impl OtherFrom<SchemaId> for String {
    fn other_from(value: SchemaId) -> Self {
        value.0
    }
}
