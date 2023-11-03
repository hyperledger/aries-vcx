use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use futures::StreamExt;
use indy_credx::{
    issuer, prover,
    tails::{TailsFileReader, TailsFileWriter},
    types::{
        AttributeNames, Credential, CredentialDefinition, CredentialDefinitionConfig,
        CredentialDefinitionId, CredentialDefinitionPrivate, CredentialKeyCorrectnessProof,
        CredentialOffer, CredentialRequest, CredentialRequestMetadata, CredentialRevocationConfig,
        CredentialRevocationState, CredentialValues, DidValue, IssuanceType, LinkSecret,
        PresentCredentials, Presentation, PresentationRequest, RegistryType, RevocationRegistry,
        RevocationRegistryDefinition, RevocationRegistryDefinitionPrivate, RevocationRegistryDelta,
        RevocationRegistryId, Schema, SchemaId, SignatureType,
    },
    verifier,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{VcIssuer, VcProver, VcVerifier};
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
        for<'b> Self::RevRegInfo: WalletRecord<W, RecordIdRef<'b> = &'b Self::RevRegId>,
    {
        let issuer_did = issuer_did.to_owned().into();

        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_owned()));

        let cred_def = wallet.get(cred_def_id.as_ref()).await?;

        let rev_reg_id = issuer::make_revocation_registry_id(
            &issuer_did,
            &cred_def,
            tag,
            RegistryType::CL_ACCUM,
        )?;

        let res_rev_reg = wallet.get(rev_reg_id.as_ref()).await;
        let res_rev_reg_def = wallet.get(rev_reg_id.as_ref()).await;

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
            .add(rev_reg_info.into_wallet_record(&rev_reg_id)?)
            .await?;
        wallet
            .add(rev_reg_def.as_wallet_record(&rev_reg_id)?)
            .await?;
        wallet
            .add(rev_reg_def_priv.into_wallet_record(&rev_reg_id)?)
            .await?;
        wallet.add(rev_reg.as_wallet_record(&rev_reg_id)?).await?;

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
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        for<'a> Self::Schema: WalletRecord<W, RecordIdRef<'a> = &'a Self::SchemaId>,
        for<'a> Self::SchemaId: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        for<'a> Self::CredDef: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        for<'a> Self::CredDefPriv: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        for<'a> Self::CredKeyProof: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
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
        if let Ok(cred_def) = wallet.get(cred_def_id.as_ref()).await {
            return Ok((cred_def_id, cred_def));
        }

        // Otherwise, create cred def
        let (cred_def, cred_def_priv, cred_key_correctness_proof) =
            issuer::create_credential_definition(&issuer_did, &schema, tag, sig_type, config)?;

        wallet.add(cred_def.as_wallet_record(&cred_def_id)?).await?;

        wallet
            .add(cred_def_priv.into_wallet_record(&cred_def_id)?)
            .await?;

        wallet
            .add(cred_key_correctness_proof.into_wallet_record(&cred_def_id)?)
            .await?;

        let schema_id = schema.id().clone();
        wallet.add(schema.into_wallet_record(&schema_id)?).await?;

        wallet
            .add(schema_id.into_wallet_record(&cred_def_id)?)
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
        Self::CredDefId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::SchemaId: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        Self::CredDef: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
        Self::CredKeyProof: WalletRecord<W, RecordIdRef<'a> = &'a Self::CredDefId>,
    {
        let cred_def: CredentialDefinition = wallet.get(cred_def_id.as_ref()).await?;
        let correctness_proof: CredentialKeyCorrectnessProof =
            wallet.get(cred_def_id.as_ref()).await?;
        let schema_id: SchemaId = wallet.get(cred_def_id.as_ref()).await?;

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
        Self::RevRegInfo: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
    {
        // TODO: Might need to qualify with offer method or something - look into how vdrtools
        // does     // it
        let cred_def_id = &cred_offer.cred_def_id;

        let cred_def = wallet.get(cred_def_id.as_ref()).await?;

        let cred_def_private = wallet.get(cred_def_id.as_ref()).await?;

        let mut revocation_config_parts = match rev_reg_id {
            Some(rev_reg_id) => {
                let rev_reg_def = wallet.get(rev_reg_id.as_ref()).await?;

                let rev_reg_def_priv = wallet.get(rev_reg_id.as_ref()).await?;

                let rev_reg = wallet.get(rev_reg_id.as_ref()).await?;
                let rev_reg_info: RevocationRegistryInfo = wallet.get(rev_reg_id.as_ref()).await?;

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
                .update(rev_reg.into_wallet_record(rev_reg_id)?)
                .await?;

            wallet
                .update(rev_reg_info.into_wallet_record(rev_reg_id)?)
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

    // TODO - FUTURE - think about moving this to somewhere else, as it aggregates other calls
    // (not // PURE Anoncreds)
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
        for<'b> Self::CredDef: WalletRecord<W, RecordIdRef<'b> = &'b Self::CredDefId>,
    {
        let rev_reg = wallet.get(rev_reg_id.as_ref()).await?;

        let rev_reg_def = wallet.get(rev_reg_id.as_ref()).await?;

        let rev_reg_priv = wallet.get(rev_reg_id.as_ref()).await?;

        let mut rev_reg_info: RevocationRegistryInfo = wallet.get(rev_reg_id.as_ref()).await?;

        let (issuance_type, cred_def_id) = match &rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => {
                (r.value.issuance_type, &r.cred_def_id)
            }
        };

        let cred_def = wallet.get(cred_def_id.as_ref()).await?;

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
            .update(rev_reg.into_wallet_record(rev_reg_id)?)
            .await?;
        wallet
            .update(rev_reg_info.into_wallet_record(rev_reg_id)?)
            .await?;

        let record = rev_reg_delta.into_wallet_record(rev_reg_id)?;
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
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
    {
        let res_rev_reg_delta = wallet.get(rev_reg_id.as_ref()).await;

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
        Self::RevRegId: AsRef<<W as Wallet>::RecordIdRef>,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::RevRegDelta: WalletRecord<W, RecordIdRef<'a> = &'a Self::RevRegId>,
    {
        if self
            .get_revocation_delta(wallet, rev_reg_id)
            .await?
            .is_some()
        {
            wallet
                .delete::<RevocationRegistryDelta>(rev_reg_id.as_ref())
                .await?;
        }

        Ok(())
    }
}

pub struct IndyCredxProver;

#[async_trait]
impl VcProver for IndyCredxProver {
    type Presentation = Presentation;
    type PresentationRequest = PresentationRequest;

    type SchemaId = SchemaId;
    type Schema = Schema;

    type CredDefId = CredentialDefinitionId;
    type CredDef = CredentialDefinition;

    type CredId = String; // does it need a type?
    type Cred = Credential;
    type CredRevId = u32;
    type CredRevState = CredentialRevocationState;
    type CredRevStateParts = (RevocationRegistryDefinition, RevocationRegistryDelta);

    type RevRegId = RevocationRegistryId;
    type RevRegDef = RevocationRegistryDefinition;
    type RevStates = String; // needs a type

    type CredReq = CredentialRequest;
    type CredReqMeta = CredentialRequestMetadata;
    type CredOffer = CredentialOffer;

    type LinkSecretId = String; // does it need a type?
    type LinkSecret = LinkSecret;

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
        Self::LinkSecret: WalletRecord<W>,
    {
        let requested_credentials: Value = serde_json::from_str(requested_credentials)?;
        let requested_attributes = requested_credentials
            .get("requested_attributes")
            .ok_or_else(bad_json)?;

        let requested_predicates = requested_credentials
            .get("requested_predicates")
            .ok_or_else(bad_json)?;

        let self_attested_attributes = requested_credentials.get("self_attested_attributes");

        let mut present_credentials: PresentCredentials = PresentCredentials::new();

        let mut proof_details_by_cred_id: HashMap<
            String,
            (
                Credential,
                Option<u64>,
                Option<CredentialRevocationState>,
                Vec<(String, bool)>,
                Vec<String>,
            ),
        > = HashMap::new();

        let req_attrs = requested_attributes.as_object().ok_or_else(bad_json)?;

        // add cred data and referent details for each requested attribute
        for (reft, detail) in req_attrs.iter() {
            let _cred_id = detail.get("cred_id").ok_or_else(bad_json)?;
            let cred_id = _cred_id.as_str().ok_or_else(bad_json)?.to_owned();

            let revealed = detail
                .get("revealed")
                .and_then(Value::as_bool)
                .ok_or_else(bad_json)?;

            if let Some((_, _, _, req_attr_refts_revealed, _)) =
                proof_details_by_cred_id.get_mut(&cred_id)
            {
                // mapping made for this credential already, add reft and its revealed status
                req_attr_refts_revealed.push((reft.to_string(), revealed));
            } else {
                let credential = wallet.get(cred_id.as_ref()).await?;

                let (timestamp, rev_state) =
                    get_rev_state(&cred_id, &credential, detail, rev_states)?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential,
                        timestamp,
                        rev_state,
                        vec![(reft.to_string(), revealed)],
                        vec![],
                    ),
                );
            }
        }

        // add cred data and referent details for each requested predicate
        for (reft, detail) in requested_predicates
            .as_object()
            .ok_or_else(bad_json)?
            .iter()
        {
            let _cred_id = detail.get("cred_id").ok_or_else(bad_json)?;
            let cred_id = _cred_id.as_str().ok_or_else(bad_json)?.to_owned();

            if let Some((_, _, _, _, req_preds_refts)) = proof_details_by_cred_id.get_mut(&cred_id)
            {
                // mapping made for this credential already, add reft
                req_preds_refts.push(reft.to_string());
            } else {
                let credential = wallet.get(cred_id.as_ref()).await?;

                let (timestamp, rev_state) =
                    get_rev_state(&cred_id, &credential, detail, rev_states)?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential,
                        timestamp,
                        rev_state,
                        vec![],
                        vec![reft.to_string()],
                    ),
                );
            }
        }

        // add all accumulated requested attributes and requested predicates to credx
        // [PresentCredential] object
        for (
            _cred_id,
            (credential, timestamp, rev_state, req_attr_refts_revealed, req_preds_refts),
        ) in proof_details_by_cred_id.iter()
        {
            let mut add_cred =
                present_credentials.add_credential(credential, *timestamp, rev_state.as_ref());

            for (referent, revealed) in req_attr_refts_revealed {
                add_cred.add_requested_attribute(referent, *revealed);
            }

            for referent in req_preds_refts {
                add_cred.add_requested_predicate(referent);
            }
        }

        // create self_attested by iterating thru self_attested_value
        let self_attested = if let Some(self_attested_value) = self_attested_attributes {
            let mut self_attested_map: HashMap<String, String> = HashMap::new();
            let self_attested_obj = self_attested_value
                .as_object()
                .ok_or_else(bad_json)?
                .clone();
            let self_attested_iter = self_attested_obj.iter();
            for (k, v) in self_attested_iter {
                self_attested_map
                    .insert(k.to_string(), v.as_str().ok_or_else(bad_json)?.to_string());
            }

            if self_attested_map.is_empty() {
                None
            } else {
                Some(self_attested_map)
            }
        } else {
            None
        };

        let link_secret = wallet.get(link_secret_id.as_ref()).await?;

        let presentation = prover::create_presentation(
            &pres_req,
            present_credentials,
            self_attested,
            &link_secret,
            &hashmap_as_ref(schemas),
            &hashmap_as_ref(cred_defs),
        )?;

        Ok(presentation)
    }

    async fn get_credentials_for_proof_req<W>(
        &self,
        wallet: &W,
        pres_request: Self::PresentationRequest,
    ) -> VcxCoreResult<String>
    // Needs a proper return type
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::SearchFilter: From<String>,
        Self::Cred: WalletRecord<W, RecordId = String>,
    {
        let (mut requested_attributes, mut requested_predicates, non_revoked) = match pres_request {
            PresentationRequest::PresentationRequestV1(pr)
            | PresentationRequest::PresentationRequestV2(pr) => (
                pr.requested_attributes,
                pr.requested_predicates,
                pr.non_revoked,
            ),
        };

        let referents: HashSet<String> = requested_attributes
            .keys()
            .map(ToOwned::to_owned)
            .chain(requested_predicates.keys().map(ToOwned::to_owned))
            .collect();

        let mut cred_by_attr: Value = json!({});

        for reft in referents {
            let (name, names) = requested_attributes
                .remove(&reft)
                .map(|a| (a.name, a.names))
                .or_else(|| {
                    requested_predicates
                        .remove(&reft)
                        .map(|p| (Some(p.name), None))
                })
                .ok_or(AriesVcxCoreError::from_msg(
                    // should not happen
                    AriesVcxCoreErrorKind::InvalidState,
                    format!("Unknown referent: {}", reft),
                ))?;

            let attr_names = match (name, names) {
                (Some(name), None) => {
                    vec![_normalize_attr_name(&name)]
                }
                (None, Some(names)) => names
                    .iter()
                    .map(String::as_str)
                    .map(_normalize_attr_name)
                    .collect(),
                _ => Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidInput,
                    "exactly one of 'name' or 'names' must be present",
                ))?,
            };

            let mut attrs = Vec::new();

            for name in attr_names {
                let attr_marker_tag_name = _format_attribute_as_marker_tag_name(&name);

                let wql_attr_query = json!({
                    attr_marker_tag_name: "1"
                });

                attrs.push(wql_attr_query);
            }

            let wql_query = json!({ "$and": attrs }).to_string();

            let mut credx_creds = Vec::new();
            let mut stream = wallet.search::<Credential>(From::from(wql_query)).await?;

            while let Some(record) = stream.next().await {
                credx_creds.push(record?);
            }

            let mut credentials_json = Vec::with_capacity(credx_creds.len());

            for (cred_id, credx_cred) in credx_creds {
                credentials_json.push(json!({
                    "cred_info": _make_cred_info(&cred_id, &credx_cred)?,
                    "interval": non_revoked
                }))
            }

            cred_by_attr["attrs"][reft] = Value::Array(credentials_json);
        }

        Ok(serde_json::to_string(&cred_by_attr)?)
    }

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        cred_rev_state_parts: Self::CredRevStateParts,
        timestamp: u64,
        cred_rev_id: Self::CredRevId,
    ) -> VcxCoreResult<Self::CredRevState> {
        let (rev_reg_def, rev_reg_delta) = cred_rev_state_parts;

        let tails_file_hash = match &rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => &r.value.tails_hash,
        };

        let mut tails_file_path = std::path::PathBuf::new();
        tails_file_path.push(tails_dir);
        tails_file_path.push(tails_file_hash);

        let tails_path = tails_file_path.to_str().ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidOption,
                "tails file is not an unicode string",
            )
        })?;

        let tails_reader = TailsFileReader::new(tails_path);
        let rev_reg_idx: u32 = cred_rev_id;

        let rev_state = prover::create_or_update_revocation_state(
            tails_reader,
            &rev_reg_def,
            &rev_reg_delta,
            rev_reg_idx,
            timestamp,
            None,
        )?;

        Ok(rev_state)
    }

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
        Self::LinkSecret: WalletRecord<W>,
    {
        let prover_did = DidValue::new(prover_did, None);
        let link_secret = wallet.get(link_secret_id.as_ref()).await?;

        let (cred_req, cred_req_metadata) = prover::create_credential_request(
            &prover_did,
            cred_def,
            &link_secret,
            link_secret_id,
            cred_offer,
        )?;

        Ok((cred_req, cred_req_metadata))
    }

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
        for<'a> Self::Cred: WalletRecord<W, RecordIdRef<'a> = Self::CredId>,
    {
        let link_secret_id = cred_req_metadata.master_secret_name.clone();
        let link_secret = wallet.get(link_secret_id.as_ref()).await?;

        prover::process_credential(cred, cred_req_metadata, &link_secret, cred_def, rev_reg_def)?;

        let schema_id = &cred.schema_id;
        let (_schema_method, schema_issuer_did, schema_name, schema_version) =
            schema_id.parts().ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidSchema,
                "Could not process credential.schema_id as parts.",
            ))?;

        let cred_def_id = &cred.cred_def_id;
        let (_cred_def_method, issuer_did, _signature_type, _schema_id, _tag) =
            cred_def_id.parts().ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidSchema,
                "Could not process credential.cred_def_id as parts.",
            ))?;

        let mut tags = json!({
            "schema_id": schema_id.0,
            "schema_issuer_did": schema_issuer_did.0,
            "schema_name": schema_name,
            "schema_version": schema_version,
            "issuer_did": issuer_did.0,
            "cred_def_id": cred_def_id.0
        });

        if let Some(rev_reg_id) = &cred.rev_reg_id {
            tags["rev_reg_id"] = serde_json::Value::String(rev_reg_id.0.to_string())
        }

        for (raw_attr_name, attr_value) in cred.values.0.iter() {
            let attr_name = _normalize_attr_name(raw_attr_name);
            // add attribute name and raw value pair
            let value_tag_name = _format_attribute_as_value_tag_name(&attr_name);
            tags[value_tag_name] = Value::String(attr_value.raw.to_string());

            // add attribute name and marker (used for checking existent)
            let marker_tag_name = _format_attribute_as_marker_tag_name(&attr_name);
            tags[marker_tag_name] = Value::String("1".to_string());
        }

        let credential_id = cred_id.map_or(Uuid::new_v4().to_string(), String::from);
        wallet
            .add(cred.as_wallet_record(credential_id.clone())?)
            .await?;

        Ok(credential_id)
    }

    /// IMPORTANT: This stores the link secret as the data structure it is, instead of
    /// storing only the String representation.
    async fn create_link_secret<W>(
        &self,
        wallet: &W,
        link_secret_id: Self::LinkSecretId,
    ) -> VcxCoreResult<()>
    where
        W: Wallet + Send + Sync,
        <W as Wallet>::RecordIdRef: Send + Sync,
        Self::LinkSecretId: AsRef<<W as Wallet>::RecordIdRef>,
        for<'a> Self::LinkSecret: WalletRecord<W, RecordIdRef<'a> = Self::LinkSecretId>,
    {
        let existing_record = wallet.get::<LinkSecret>(link_secret_id.as_ref()).await.ok();

        if existing_record.is_some() {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::DuplicationMasterSecret,
                format!(
                    "Master secret id: {} already exists in wallet.",
                    link_secret_id
                ),
            ));
        }

        let secret = prover::create_link_secret()?;
        wallet
            .add(secret.into_wallet_record(link_secret_id)?)
            .await?;

        Ok(())
    }
}

pub struct IndyCredxVerifier;

#[async_trait]
impl VcVerifier for IndyCredxVerifier {
    type PresentationRequest = PresentationRequest;
    type Presentation = Presentation;

    type SchemaId = SchemaId;
    type Schema = Schema;

    type CredDefId = CredentialDefinitionId;
    type CredDef = CredentialDefinition;

    type RevRegId = RevocationRegistryId;
    type RevRegDef = RevocationRegistryDefinition;
    type RevStates = HashMap<u64, RevocationRegistry>;

    async fn verify_proof(
        &self,
        pres_request: &Self::PresentationRequest,
        presentation: &Self::Presentation,
        schemas: &HashMap<Self::SchemaId, Self::Schema>,
        credential_defs: &HashMap<Self::CredDefId, Self::CredDef>,
        rev_reg_defs: Option<&HashMap<Self::RevRegId, Self::RevRegDef>>,
        rev_regs: Option<&HashMap<Self::RevRegId, Self::RevStates>>,
    ) -> VcxCoreResult<bool> {
        let rev_regs = if let Some(map) = rev_regs {
            let new_map = map
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|(k, v)| (*k, v)).collect()))
                .collect();

            Some(new_map)
        } else {
            None
        };
        let output = verifier::verify_presentation(
            presentation,
            pres_request,
            &hashmap_as_ref(schemas),
            &hashmap_as_ref(credential_defs),
            rev_reg_defs.map(hashmap_as_ref).as_ref(),
            rev_regs.as_ref(),
        )?;

        #[cfg(feature = "legacy_proof")]
        let output = output
            || verifier::verify_presentation_legacy(
                presentation,
                pres_request,
                &hashmap_as_ref(schemas),
                &hashmap_as_ref(credential_defs),
                rev_reg_defs.map(hashmap_as_ref).as_ref(),
                rev_regs.as_ref(),
            )?;

        Ok(output)
    }

    async fn generate_nonce(&self) -> VcxCoreResult<String> {
        verifier::generate_nonce()
            .map_err(From::from)
            .map(|v| v.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}

// common transformation requirement in credx
fn hashmap_as_ref<T, U>(map: &HashMap<T, U>) -> HashMap<T, &U>
where
    T: std::hash::Hash,
    T: std::cmp::Eq,
    T: std::clone::Clone,
{
    let mut new_map: HashMap<T, &U> = HashMap::new();
    for (k, v) in map.iter() {
        new_map.insert(k.clone(), v);
    }

    new_map
}

fn get_rev_state(
    cred_id: &str,
    credential: &Credential,
    detail: &Value,
    rev_states: Option<&HashMap<RevocationRegistryId, String>>,
) -> VcxCoreResult<(Option<u64>, Option<CredentialRevocationState>)> {
    let timestamp = detail
        .get("timestamp")
        .and_then(|timestamp| timestamp.as_u64());
    let cred_rev_reg_id = credential.rev_reg_id.as_ref().map(|id| id.0.to_string());
    let rev_state = if let (Some(timestamp), Some(cred_rev_reg_id)) = (timestamp, cred_rev_reg_id) {
        let rev_state =
            rev_states.and_then(|m| m.get(&RevocationRegistryId(cred_rev_reg_id.clone())));
        let rev_state = rev_state.ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!(
                "No revocation states provided for credential '{}' with rev_reg_id '{}'",
                cred_id, cred_rev_reg_id
            ),
        ))?;

        let rev_state: Value = serde_json::from_str(rev_state)?;
        let rev_state =
            rev_state
                .get(&timestamp.to_string())
                .ok_or(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidJson,
                    format!(
                        "No revocation states provided for credential '{}' with rev_reg_id '{}' \
                         at timestamp '{}'",
                        cred_id, cred_rev_reg_id, timestamp
                    ),
                ))?;

        let rev_state: CredentialRevocationState = serde_json::from_value(rev_state.clone())?;
        Some(rev_state)
    } else {
        None
    };

    Ok((timestamp, rev_state))
}

fn _normalize_attr_name(name: &str) -> String {
    // "name": string, // attribute name, (case insensitive and ignore spaces)
    name.replace(' ', "").to_lowercase()
}

fn bad_json() -> AriesVcxCoreError {
    AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, "STOP USING JSON!!!!")
}

fn _format_attribute_as_value_tag_name(attribute_name: &str) -> String {
    format!("attr::{attribute_name}::value")
}

fn _format_attribute_as_marker_tag_name(attribute_name: &str) -> String {
    format!("attr::{attribute_name}::marker")
}

fn _make_cred_info(credential_id: &str, cred: &Credential) -> VcxCoreResult<Value> {
    let cred_sig = serde_json::to_value(&cred.signature)?;

    let rev_info = cred_sig.get("r_credential");

    let schema_id = &cred.schema_id.0;
    let cred_def_id = &cred.cred_def_id.0;
    let rev_reg_id = cred.rev_reg_id.as_ref().map(|x| x.0.to_string());
    let cred_rev_id = rev_info.and_then(|x| x.get("i")).and_then(|i| {
        i.as_str()
            .map(|str_i| str_i.to_string())
            .or(i.as_i64().map(|int_i| int_i.to_string()))
    });

    let mut attrs = json!({});
    for (x, y) in cred.values.0.iter() {
        attrs[x] = Value::String(y.raw.to_string());
    }

    let val = json!({
        "referent": credential_id,
        "schema_id": schema_id,
        "cred_def_id": cred_def_id,
        "rev_reg_id": rev_reg_id,
        "cred_rev_id": cred_rev_id,
        "attrs": attrs
    });

    Ok(val)
}

/// Just proving that stuff compiles
#[cfg(test)]
#[allow(unused, clippy::all)]
#[test]
fn stuff() {
    use indy_api_types::WalletHandle;

    use crate::{wallet::indy::IndySdkWallet, wallet2::vdrtools::IndyWalletId};

    // SAFETY:
    // We're only changing types, but the layout is the same because of #[repr(transparent)].
    // This is because we can't implement AsRef<str> for the remote types.
    //
    // Indy-credx should implement `AsRef<str>` for their ID types, which would make
    // the usage of this redundant. And they should drop the `Deref<str>` thing as that's
    // meant for pointer types, not to mimic inheritance.
    //
    // Another popular example: https://docs.rs/serde_json/latest/src/serde_json/raw.rs.html#121-124
    impl AsRef<IndyWalletId> for RevocationRegistryId {
        fn as_ref(&self) -> &IndyWalletId {
            unsafe { std::mem::transmute::<&str, &IndyWalletId>(self.0.as_str()) }
        }
    }

    impl AsRef<IndyWalletId> for CredentialDefinitionId {
        fn as_ref(&self) -> &IndyWalletId {
            unsafe { std::mem::transmute::<&str, &IndyWalletId>(self.0.as_str()) }
        }
    }

    impl WalletRecord<IndySdkWallet> for Credential {
        const RECORD_TYPE: &'static str = "cred";

        type RecordIdRef<'a> = &'a str;
        type RecordId = String;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl WalletRecord<IndySdkWallet> for CredentialDefinition {
        const RECORD_TYPE: &'static str = "cred";

        type RecordIdRef<'a> = &'a RevocationRegistryId;
        type RecordId = RevocationRegistryId;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl WalletRecord<IndySdkWallet> for RevocationRegistry {
        const RECORD_TYPE: &'static str = "rev";

        type RecordIdRef<'a> = &'a RevocationRegistryId;
        type RecordId = RevocationRegistryId;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl WalletRecord<IndySdkWallet> for RevocationRegistryDefinition {
        const RECORD_TYPE: &'static str = "rev";

        type RecordIdRef<'a> = &'a RevocationRegistryId;
        type RecordId = RevocationRegistryId;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl WalletRecord<IndySdkWallet> for RevocationRegistryDefinitionPrivate {
        const RECORD_TYPE: &'static str = "rev";

        type RecordIdRef<'a> = &'a RevocationRegistryId;
        type RecordId = RevocationRegistryId;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl WalletRecord<IndySdkWallet> for RevocationRegistryDelta {
        const RECORD_TYPE: &'static str = "rev";

        type RecordIdRef<'a> = &'a RevocationRegistryId;
        type RecordId = RevocationRegistryId;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl WalletRecord<IndySdkWallet> for RevocationRegistryInfo {
        const RECORD_TYPE: &'static str = "rev";

        type RecordIdRef<'a> = &'a RevocationRegistryId;
        type RecordId = RevocationRegistryId;

        fn into_wallet_record(
            self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn as_wallet_record(
            &self,
            id: Self::RecordIdRef<'_>,
        ) -> VcxCoreResult<<IndySdkWallet as Wallet>::Record> {
            todo!()
        }

        fn from_wallet_record(
            record: <IndySdkWallet as Wallet>::Record,
        ) -> VcxCoreResult<(Self::RecordId, Self)>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    let issuer = IndyCredxIssuer;
    let prover = IndyCredxProver;
    let wallet = IndySdkWallet::new(WalletHandle(0));

    let rev_reg_id = RevocationRegistryId(String::from("blabla"));
    let cred_def_id = CredentialDefinitionId(String::from("blabla"));

    let pres_request = serde_json::from_str("").unwrap();

    async {
        issuer.get_revocation_delta(&wallet, &rev_reg_id).await;
        issuer
            .create_and_store_revoc_reg(
                &wallet,
                "bla",
                &cred_def_id,
                "test",
                10,
                "404_no_tags_found",
            )
            .await;

        prover.get_credentials_for_proof_req(&wallet, pres_request)
    };
}
