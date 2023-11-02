use std::collections::HashSet;

use async_trait::async_trait;
use indy_credx::{
    issuer,
    tails::TailsFileWriter,
    types::{
        AttributeNames, Credential, CredentialDefinition, CredentialDefinitionConfig,
        CredentialDefinitionId, CredentialDefinitionPrivate, CredentialKeyCorrectnessProof,
        CredentialOffer, CredentialRequest, CredentialRevocationConfig, CredentialValues, DidValue,
        IssuanceType, RegistryType, RevocationRegistry, RevocationRegistryDefinition,
        RevocationRegistryDefinitionPrivate, RevocationRegistryDelta, RevocationRegistryId, Schema,
        SchemaId, SignatureType,
    },
};
use serde::{Deserialize, Serialize};

use super::VcIssuer;
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
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

    async fn create_and_store_revoc_reg<'a, W>(
        &self,
        wallet: &W,
        issuer_did: &str,
        cred_def_id: &'a Self::CredDefId,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(Self::RevRegId, Self::RevRegDef, Self::RevReg)>
    where
        W: Wallet + Send + Sync,
        for<'b> <W as Wallet>::RecordId:
            From<&'a Self::CredDefId> + From<&'b Self::RevRegId> + Send + Sync,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevReg: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>,
    {
        let issuer_did = issuer_did.to_owned().into();

        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_owned()));

        let cred_def = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        let rev_reg_id = issuer::make_revocation_registry_id(
            &issuer_did,
            &cred_def,
            tag,
            RegistryType::CL_ACCUM,
        )?;

        let res_rev_reg = wallet.get(&W::RecordId::from(&rev_reg_id)).await;
        let res_rev_reg_def = wallet.get(&W::RecordId::from(&rev_reg_id)).await;

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
            .add(rev_reg_info.into_wallet_record(W::RecordId::from(&rev_reg_id))?)
            .await?;
        wallet
            .add(rev_reg_def.as_wallet_record(W::RecordId::from(&rev_reg_id))?)
            .await?;
        wallet
            .add(rev_reg_def_priv.into_wallet_record(W::RecordId::from(&rev_reg_id))?)
            .await?;
        wallet
            .add(rev_reg.as_wallet_record(W::RecordId::from(&rev_reg_id))?)
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
        for<'a> <W as Wallet>::RecordId:
            From<&'a Self::CredDefId> + From<&'a Self::SchemaId> + Send + Sync,
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
        if let Ok(cred_def) = wallet.get(&W::RecordId::from(&cred_def_id)).await {
            return Ok((cred_def_id, cred_def));
        }

        // Otherwise, create cred def
        let (cred_def, cred_def_priv, cred_key_correctness_proof) =
            issuer::create_credential_definition(&issuer_did, &schema, tag, sig_type, config)?;

        wallet
            .add(cred_def.as_wallet_record(W::RecordId::from(&cred_def_id))?)
            .await?;

        wallet
            .add(cred_def_priv.into_wallet_record(W::RecordId::from(&cred_def_id))?)
            .await?;

        wallet
            .add(cred_key_correctness_proof.into_wallet_record(W::RecordId::from(&cred_def_id))?)
            .await?;

        let schema_id = schema.id().clone();
        wallet
            .add(schema.into_wallet_record(W::RecordId::from(&schema_id))?)
            .await?;

        wallet
            .add(schema_id.into_wallet_record(W::RecordId::from(&cred_def_id))?)
            .await?;

        // Return the ID and the cred def
        Ok((cred_def_id, cred_def))
    }

    async fn create_credential_offer<'a, W>(
        &self,
        wallet: &W,
        cred_def_id: &'a Self::CredDefId,
    ) -> VcxCoreResult<Self::CredOffer>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordId: From<&'a Self::CredDefId> + Send + Sync,
        Self::SchemaId: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredKeyProof: WalletRecord<W, RecordId = W::RecordId>,
    {
        let cred_def = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        let correctness_proof = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        let schema_id = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        // If cred_def contains schema ID, why take it as an argument here...?
        let offer = issuer::create_credential_offer(&schema_id, &cred_def, &correctness_proof)?;

        Ok(offer)
    }

    async fn create_credential<'a, W>(
        &self,
        wallet: &W,
        cred_offer: Self::CredOffer,
        cred_request: Self::CredReq,
        cred_values: Self::CredValues,
        rev_reg_id: Option<&'a Self::RevRegId>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(Self::Cred, Option<Self::CredRevId>)>
    where
        W: Wallet + Send + Sync,
        for<'b> <W as Wallet>::RecordId:
            From<&'b Self::CredDefId> + From<&'a Self::RevRegId> + Send + Sync,
        Self::Schema: WalletRecord<W, RecordId = W::RecordId>,
        Self::SchemaId: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevReg: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredKeyProof: WalletRecord<W, RecordId = W::RecordId>,
    {
        // TODO: Might need to qualify with offer method or something - look into how vdrtools does
        // it
        let cred_def_id = &cred_offer.cred_def_id;

        let cred_def = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        let cred_def_private = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        let mut revocation_config_parts = match rev_reg_id {
            Some(rev_reg_id) => {
                let rev_reg_def = wallet.get(&W::RecordId::from(rev_reg_id)).await?;

                let rev_reg_def_priv = wallet.get(&W::RecordId::from(rev_reg_id)).await?;

                let rev_reg = wallet.get(&W::RecordId::from(rev_reg_id)).await?;
                let rev_reg_info: RevocationRegistryInfo =
                    wallet.get(&W::RecordId::from(rev_reg_id)).await?;

                Some((rev_reg_def, rev_reg_def_priv, rev_reg, rev_reg_info))
            }
            None => {
                warn!(
                    "Missing revocation config params: tails_dir: {tails_dir:?} - {rev_reg_id:?}; \
                     Issuing non revokable credential"
                );
                None
            }
        };

        let revocation_config = match &mut revocation_config_parts {
            Some((rev_reg_def, rev_reg_def_priv, rev_reg, rev_reg_info)) => {
                rev_reg_info.curr_id += 1;

                let RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def_v1) =
                    rev_reg_def;

                if rev_reg_info.curr_id > rev_reg_def_v1.value.max_cred_num {
                    return Err(AriesVcxCoreError::from_msg(
                        AriesVcxCoreErrorKind::ActionNotSupported,
                        "The revocation registry is full",
                    ));
                }

                if rev_reg_def_v1.value.issuance_type == IssuanceType::ISSUANCE_ON_DEMAND {
                    rev_reg_info.used_ids.insert(rev_reg_info.curr_id);
                }

                let revocation_config = CredentialRevocationConfig {
                    reg_def: rev_reg_def,
                    reg_def_private: rev_reg_def_priv,
                    registry: rev_reg,
                    registry_idx: rev_reg_info.curr_id,
                    registry_used: &rev_reg_info.used_ids,
                };

                Some(revocation_config)
            }
            None => None,
        };

        let (cred, rev_reg, _) = issuer::create_credential(
            &cred_def,
            &cred_def_private,
            &cred_offer,
            &cred_request,
            cred_values,
            revocation_config,
        )?;

        let cred_rev_id = if let (Some(rev_reg_id), Some(rev_reg), Some((_, _, _, rev_reg_info))) =
            (rev_reg_id, rev_reg, revocation_config_parts)
        {
            let cred_rev_id = rev_reg_info.curr_id;

            wallet
                .update(rev_reg.into_wallet_record(W::RecordId::from(rev_reg_id))?)
                .await?;

            wallet
                .update(rev_reg_info.into_wallet_record(W::RecordId::from(rev_reg_id))?)
                .await?;

            Some(cred_rev_id)
        } else {
            None
        };

        Ok((cred, cred_rev_id))
    }

    async fn create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: Self::SchemaAttrNames,
    ) -> VcxCoreResult<(Self::SchemaId, Self::Schema)> {
        let origin_did = DidValue::new(issuer_did, None);
        let schema = issuer::create_schema(&origin_did, name, version, attrs, None)?;
        Ok((schema.id().clone(), schema))
    }

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls (not
    // PURE Anoncreds)
    async fn revoke_credential<'a, W>(
        &self,
        wallet: &W,
        rev_reg_id: &'a Self::RevRegId,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        for<'b> <W as Wallet>::RecordId:
            From<&'b Self::CredDefId> + From<&'a Self::RevRegId> + Send + Sync,
        Self::RevRegDef: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDefPriv: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevReg: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegInfo: WalletRecord<W, RecordId = W::RecordId>,
        Self::RevRegDelta: WalletRecord<W, RecordId = W::RecordId>,
        Self::CredDef: WalletRecord<W, RecordId = W::RecordId>,
    {
        let rev_reg = wallet.get(&W::RecordId::from(rev_reg_id)).await?;

        let rev_reg_def = wallet.get(&W::RecordId::from(rev_reg_id)).await?;

        let rev_reg_priv = wallet.get(&W::RecordId::from(rev_reg_id)).await?;

        let mut rev_reg_info: RevocationRegistryInfo =
            wallet.get(&W::RecordId::from(rev_reg_id)).await?;

        let (issuance_type, cred_def_id) = match &rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => {
                (r.value.issuance_type, &r.cred_def_id)
            }
        };

        let cred_def = wallet.get(&W::RecordId::from(cred_def_id)).await?;

        match issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.remove(&cred_rev_id) {
                    return Err(AriesVcxCoreError::from_msg(
                        AriesVcxCoreErrorKind::InvalidInput,
                        format!(
                            "Revocation id: {:?} not found in RevocationRegistry",
                            cred_rev_id
                        ),
                    ));
                };
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.insert(cred_rev_id) {
                    return Err(AriesVcxCoreError::from_msg(
                        AriesVcxCoreErrorKind::InvalidInput,
                        format!(
                            "Revocation id: {:?} not found in RevocationRegistry",
                            cred_rev_id
                        ),
                    ));
                }
            }
        };

        let (rev_reg, new_rev_reg_delta) = issuer::revoke_credential(
            &cred_def,
            &rev_reg_def,
            &rev_reg_priv,
            &rev_reg,
            cred_rev_id,
        )?;

        let old_rev_reg_delta = self.get_revocation_delta(wallet, rev_reg_id).await?;

        let rev_reg_delta = old_rev_reg_delta
            .as_ref()
            .map(|rev_reg_delta| {
                issuer::merge_revocation_registry_deltas(rev_reg_delta, &new_rev_reg_delta)
            })
            .transpose()?
            .unwrap_or(new_rev_reg_delta);

        wallet
            .update(rev_reg.into_wallet_record(W::RecordId::from(rev_reg_id))?)
            .await?;
        wallet
            .update(rev_reg_info.into_wallet_record(W::RecordId::from(rev_reg_id))?)
            .await?;

        let record = rev_reg_delta.into_wallet_record(W::RecordId::from(rev_reg_id))?;
        match old_rev_reg_delta {
            Some(_) => wallet.update(record).await?,
            None => wallet.add(record).await?,
        }

        Ok(())
    }

    async fn get_revocation_delta<'a, W>(
        &self,
        wallet: &W,
        rev_reg_id: &'a Self::RevRegId,
    ) -> VcxCoreResult<Option<Self::RevRegDelta>>
    where
        W: Wallet + Send + Sync,
        for<'b> <W as Wallet>::RecordId:
            From<&'b Self::CredDefId> + From<&'a Self::RevRegId> + Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordId = W::RecordId>,
    {
        let res_rev_reg_delta = wallet.get(&W::RecordId::from(rev_reg_id)).await;

        if let Err(err) = &res_rev_reg_delta {
            warn!(
                "get_rev_reg_delta >> Unable to get rev_reg_delta cache for rev_reg_id: {}, \
                 error: {}",
                rev_reg_id, err
            );
        }

        Ok(res_rev_reg_delta.ok())
    }

    async fn clear_revocation_delta<'a, W>(
        &self,
        wallet: &W,
        rev_reg_id: &'a Self::RevRegId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        for<'b> <W as Wallet>::RecordId:
            From<&'b Self::CredDefId> + From<&'a Self::RevRegId> + Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordId = W::RecordId>,
    {
        if self
            .get_revocation_delta(wallet, rev_reg_id)
            .await?
            .is_some()
        {
            wallet
                .delete::<RevocationRegistryDelta>(&W::RecordId::from(rev_reg_id))
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}
