mod type_conversion;

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anoncreds_types::data_types::ledger::schema::Schema;
use async_trait::async_trait;
use credx::{
    anoncreds_clsignatures::{bn::BigNumber, LinkSecret as ClLinkSecret},
    tails::{TailsFileReader, TailsFileWriter},
    types::{
        Credential as CredxCredential, CredentialDefinition, CredentialDefinitionId,
        CredentialOffer, CredentialRequestMetadata, CredentialRevocationConfig,
        CredentialRevocationState, DidValue, IssuanceType, LinkSecret, PresentCredentials,
        Presentation, PresentationRequest, RegistryType, RevocationRegistry,
        RevocationRegistryDefinition, RevocationRegistryDelta, RevocationRegistryId, Schema as CredxSchema,
        SchemaId, SignatureType,
    },
};
use indy_credx as credx;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use type_conversion::Convert;
use super::base_anoncreds::BaseAnonCreds;
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    utils::{
        constants::ATTRS,
        json::{AsTypeOrDeserializationError, TryGetIndex},
    },
    wallet::{
        base_wallet::{record::Record, search_filter::SearchFilter, BaseWallet, RecordWallet},
        entry_tags::EntryTags,
    },
};

pub const CATEGORY_LINK_SECRET: &str = "VCX_LINK_SECRET";

pub const CATEGORY_CREDENTIAL: &str = "VCX_CREDENTIAL";
pub const CATEGORY_CRED_DEF: &str = "VCX_CRED_DEF";
pub const CATEGORY_CRED_KEY_CORRECTNESS_PROOF: &str = "VCX_CRED_KEY_CORRECTNESS_PROOF";
pub const CATEGORY_CRED_DEF_PRIV: &str = "VCX_CRED_DEF_PRIV";
pub const CATEGORY_CRED_SCHEMA: &str = "VCX_CRED_SCHEMA";

// Category used for mapping a cred_def_id to a schema_id
pub const CATEGORY_CRED_MAP_SCHEMA_ID: &str = "VCX_CRED_MAP_SCHEMA_ID";

pub const CATEGORY_REV_REG: &str = "VCX_REV_REG";
pub const CATEGORY_REV_REG_DELTA: &str = "VCX_REV_REG_DELTA";
pub const CATEGORY_REV_REG_INFO: &str = "VCX_REV_REG_INFO";
pub const CATEGORY_REV_REG_DEF: &str = "VCX_REV_REG_DEF";
pub const CATEGORY_REV_REG_DEF_PRIV: &str = "VCX_REV_REG_DEF_PRIV";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}

/// Adapter used so that credx does not depend strictly on the vdrtools-wallet
/// Will get removed when the wallet and anoncreds interfaces are de-coupled.
#[derive(Debug)]
struct WalletAdapter(Arc<dyn BaseWallet>);

#[async_trait]
impl RecordWallet for WalletAdapter {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        self.0.add_record(record).await
    }

    async fn get_record(&self, category: &str, name: &str) -> VcxCoreResult<Record> {
        self.0.get_record(category, name).await
    }

    async fn update_record_tags(
        &self,
        category: &str,
        name: &str,
        new_tags: EntryTags,
    ) -> VcxCoreResult<()> {
        self.0.update_record_tags(category, name, new_tags).await
    }

    async fn update_record_value(
        &self,
        category: &str,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        self.0.update_record_value(category, name, new_value).await
    }

    async fn delete_record(&self, category: &str, name: &str) -> VcxCoreResult<()> {
        self.0.delete_record(category, name).await
    }

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        self.0.search_record(category, search_filter).await
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IndyCredxAnonCreds;

impl IndyCredxAnonCreds {
    async fn get_wallet_record_value<T>(
        wallet: &impl BaseWallet,
        category: &str,
        id: &str,
    ) -> VcxCoreResult<T>
    where
        T: DeserializeOwned,
    {
        let str_record = wallet.get_record(category, id).await?;
        serde_json::from_str(str_record.value()).map_err(From::from)
    }

    async fn get_link_secret(
        wallet: &impl BaseWallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<LinkSecret> {
        let record = wallet
            .get_record(CATEGORY_LINK_SECRET, link_secret_id)
            .await?;

        let ms_bn: BigNumber = BigNumber::from_dec(record.value()).map_err(|err| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::UrsaError,
                format!(
                    "Failed to create BigNumber, UrsaErrorKind: {:?}",
                    err.kind()
                ),
            )
        })?;
        let ursa_ms: ClLinkSecret = serde_json::from_value(json!({ "ms": ms_bn }))?;

        Ok(LinkSecret { value: ursa_ms })
    }

    async fn _get_credential(
        wallet: &impl BaseWallet,
        credential_id: &str,
    ) -> VcxCoreResult<CredxCredential> {
        let cred_record = wallet
            .get_record(CATEGORY_CREDENTIAL, credential_id)
            .await?;

        let credential: CredxCredential = serde_json::from_str(cred_record.value())?;

        Ok(credential)
    }

    async fn _get_credentials(
        wallet: &impl BaseWallet,
        wql: &str,
    ) -> VcxCoreResult<Vec<(String, CredxCredential)>> {
        let records = wallet
            .search_record(
                CATEGORY_CREDENTIAL,
                Some(SearchFilter::JsonFilter(wql.into())),
            )
            .await?;

        let id_cred_tuple_list: VcxCoreResult<Vec<(String, CredxCredential)>> = records
            .into_iter()
            .map(|record| {
                let credential: CredxCredential = serde_json::from_str(record.value())?;

                Ok((record.name().into(), credential))
            })
            .collect();

        id_cred_tuple_list
    }

    async fn _get_credentials_for_proof_req_for_attr_name(
        &self,
        wallet: &impl BaseWallet,
        restrictions: Option<&Value>,
        attr_names: Vec<String>,
    ) -> VcxCoreResult<Vec<(String, CredxCredential)>> {
        let mut attrs = Vec::new();

        for name in attr_names {
            let attr_marker_tag_name = _format_attribute_as_marker_tag_name(&name);

            let wql_attr_query = json!({
                attr_marker_tag_name: "1"
            });

            attrs.push(wql_attr_query);
        }

        let restrictions = restrictions.map(|x| x.to_owned());

        let wql_query = if let Some(restrictions) = restrictions {
            match restrictions {
                Value::Array(restrictions) => {
                    let restrictions_wql = json!({ "$or": restrictions });
                    attrs.push(restrictions_wql);
                    json!({ "$and": attrs })
                }
                Value::Object(restriction) => {
                    attrs.push(Value::Object(restriction));
                    json!({ "$and": attrs })
                }
                _ => Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidInput,
                    "Invalid attribute restrictions (must be array or an object)",
                ))?,
            }
        } else {
            json!({ "$and": attrs })
        };

        let wql_query = serde_json::to_string(&wql_query)?;

        Self::_get_credentials(wallet, &wql_query).await
    }
}

#[async_trait]
impl BaseAnonCreds for IndyCredxAnonCreds {
    async fn verifier_verify_proof(
        &self,
        proof_req_json: &str,
        proof_json: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        rev_reg_defs_json: &str,
        rev_regs_json: &str,
    ) -> VcxCoreResult<bool> {
        let presentation: Presentation = serde_json::from_str(proof_json)?;
        let pres_req: PresentationRequest = serde_json::from_str(proof_req_json)?;

        let schemas_: HashMap<SchemaId, Schema> = serde_json::from_str(schemas_json)?;
        let mut schemas: HashMap<SchemaId, CredxSchema> = HashMap::new();
        for (key, value) in schemas_ {
            schemas.insert(key, value.convert(())?);
        }
        let cred_defs: HashMap<CredentialDefinitionId, CredentialDefinition> =
            serde_json::from_str(credential_defs_json)?;

        let rev_reg_defs: Option<HashMap<RevocationRegistryId, RevocationRegistryDefinition>> =
            serde_json::from_str(rev_reg_defs_json)?;

        let rev_regs: Option<HashMap<RevocationRegistryId, HashMap<u64, RevocationRegistry>>> =
            serde_json::from_str(rev_regs_json)?;
        let rev_regs: Option<HashMap<RevocationRegistryId, HashMap<u64, &RevocationRegistry>>> =
            rev_regs.as_ref().map(|regs| {
                let mut new_regs: HashMap<RevocationRegistryId, HashMap<u64, &RevocationRegistry>> =
                    HashMap::new();
                for (k, v) in regs {
                    new_regs.insert(k.clone(), hashmap_as_ref(v));
                }
                new_regs
            });

        let output = credx::verifier::verify_presentation(
            &presentation,
            &pres_req,
            &hashmap_as_ref(&schemas),
            &hashmap_as_ref(&cred_defs),
            rev_reg_defs.as_ref().map(hashmap_as_ref).as_ref(),
            rev_regs.as_ref(),
        )?;

        #[cfg(feature = "legacy_proof")]
        let output = output
            || credx::verifier::verify_presentation_legacy(
                &presentation,
                &pres_req,
                &hashmap_as_ref(&schemas),
                &hashmap_as_ref(&cred_defs),
                rev_reg_defs.as_ref().map(hashmap_as_ref).as_ref(),
                rev_regs.as_ref(),
            )?;

        Ok(output)
    }

    async fn issuer_create_and_store_revoc_reg(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxCoreResult<(String, String, String)> {
        let issuer_did = issuer_did.to_owned().into();

        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_owned()));

        let cred_def =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_DEF, cred_def_id).await?;

        let rev_reg_id = credx::issuer::make_revocation_registry_id(
            &issuer_did,
            &cred_def,
            tag,
            RegistryType::CL_ACCUM,
        )?;

        let res_rev_reg =
            Self::get_wallet_record_value(wallet, CATEGORY_REV_REG, &rev_reg_id.0).await;
        let res_rev_reg_def =
            Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF, &rev_reg_id.0).await;

        if let (Ok(rev_reg), Ok(rev_reg_def)) = (res_rev_reg, res_rev_reg_def) {
            return Ok((rev_reg_id.0, rev_reg, rev_reg_def));
        }

        let (rev_reg_def, rev_reg_def_priv, rev_reg, _rev_reg_delta) =
            credx::issuer::create_revocation_registry(
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

        let str_rev_reg_info = serde_json::to_string(&rev_reg_info)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(CATEGORY_REV_REG_INFO.to_string())
            .value(str_rev_reg_info)
            .build();
        wallet.add_record(record).await?;

        let str_rev_reg_def = serde_json::to_string(&rev_reg_def)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(CATEGORY_REV_REG_DEF.to_string())
            .value(str_rev_reg_def.clone())
            .build();
        wallet.add_record(record).await?;

        let str_rev_reg_def_priv = serde_json::to_string(&rev_reg_def_priv)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(CATEGORY_REV_REG_DEF_PRIV.to_string())
            .value(str_rev_reg_def_priv)
            .build();
        wallet.add_record(record).await?;

        let str_rev_reg = serde_json::to_string(&rev_reg)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(CATEGORY_REV_REG.to_string())
            .value(str_rev_reg.clone())
            .build();
        wallet.add_record(record).await?;

        Ok((rev_reg_id.0, str_rev_reg_def, str_rev_reg))
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &str,
        _schema_id: &str,
        schema_json: Schema,
        tag: &str,
        sig_type: Option<&str>,
        config_json: &str,
    ) -> VcxCoreResult<(String, String)> {
        let issuer_did = issuer_did.to_owned().into();
        let sig_type = sig_type
            .map(serde_json::from_str)
            .unwrap_or(Ok(SignatureType::CL))?;
        let config = serde_json::from_str(config_json)?;

        let schema_seq_no = schema_json.seq_no.clone();
        let schema = schema_json.clone().convert(())?;

        let cred_def_id = credx::issuer::make_credential_definition_id(
            &issuer_did,
            schema.id(),
            schema_seq_no,
            tag,
            sig_type,
        )?;

        // If cred def already exists, return it
        if let Ok(cred_def) =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_DEF, &cred_def_id.0).await
        {
            return Ok((cred_def_id.0, cred_def));
        }

        // Otherwise, create cred def
        let (cred_def, cred_def_priv, cred_key_correctness_proof) =
            credx::issuer::create_credential_definition(
                &issuer_did,
                &schema,
                tag,
                sig_type,
                config,
            )?;

        let str_cred_def = serde_json::to_string(&cred_def)?;
        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(CATEGORY_CRED_DEF.to_string())
            .value(str_cred_def.clone())
            .build();
        wallet.add_record(record).await?;

        let str_cred_def_priv = serde_json::to_string(&cred_def_priv)?;
        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(CATEGORY_CRED_DEF_PRIV.to_string())
            .value(str_cred_def_priv)
            .build();
        wallet.add_record(record).await?;

        let str_cred_key_proof = serde_json::to_string(&cred_key_correctness_proof)?;
        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(CATEGORY_CRED_KEY_CORRECTNESS_PROOF.to_string())
            .value(str_cred_key_proof)
            .build();
        wallet.add_record(record).await?;

        let record = Record::builder()
            .name(schema.id().to_string())
            .category(CATEGORY_CRED_SCHEMA.to_string())
            .value(serde_json::to_string(&schema_json)?)
            .build();
        let store_schema_res = wallet.add_record(record).await;

        if let Err(e) = store_schema_res {
            warn!("Storing schema {schema_json:?} failed - {e}. It's possible it is already stored.")
        }

        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(CATEGORY_CRED_MAP_SCHEMA_ID.to_string())
            .value(schema.id().0.clone())
            .build();
        wallet.add_record(record).await?;

        // Return the ID and the cred def
        Ok((cred_def_id.0.to_owned(), str_cred_def))
    }

    async fn issuer_create_credential_offer(
        &self,
        wallet: &impl BaseWallet,
        cred_def_id: &str,
    ) -> VcxCoreResult<String> {
        let cred_def =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_DEF, cred_def_id).await?;

        let correctness_proof =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_KEY_CORRECTNESS_PROOF, cred_def_id)
                .await?;

        let schema = wallet
            .get_record(CATEGORY_CRED_MAP_SCHEMA_ID, cred_def_id)
            .await?;

        let schema_id = SchemaId(schema.value().into());

        // If cred_def contains schema ID, why take it as an argument here...?
        let offer =
            credx::issuer::create_credential_offer(&schema_id, &cred_def, &correctness_proof)?;

        serde_json::to_string(&offer).map_err(From::from)
    }

    async fn issuer_create_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> VcxCoreResult<(String, Option<String>, Option<String>)> {
        let cred_offer: CredentialOffer = serde_json::from_str(cred_offer_json)?;
        let cred_request = serde_json::from_str(cred_req_json)?;
        let cred_values = serde_json::from_str(cred_values_json)?;

        // TODO: Might need to qualify with offer method or something - look into how vdrtools does
        // it
        let cred_def_id = &cred_offer.cred_def_id.0;

        let cred_def =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_DEF, cred_def_id).await?;

        let cred_def_private =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_DEF_PRIV, cred_def_id).await?;

        let mut revocation_config_parts = match &rev_reg_id {
            Some(rev_reg_id) => {
                let rev_reg_def =
                    Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF, rev_reg_id).await?;

                let rev_reg_def_priv =
                    Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF_PRIV, rev_reg_id)
                        .await?;

                let rev_reg =
                    Self::get_wallet_record_value(wallet, CATEGORY_REV_REG, rev_reg_id).await?;
                let rev_reg_info: RevocationRegistryInfo =
                    Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_INFO, rev_reg_id)
                        .await?;

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

        let (cred, rev_reg, rev_reg_delta) = credx::issuer::create_credential(
            &cred_def,
            &cred_def_private,
            &cred_offer,
            &cred_request,
            cred_values,
            revocation_config,
        )?;

        let str_rev_reg = rev_reg.as_ref().map(serde_json::to_string).transpose()?;
        let str_rev_reg_delta = rev_reg_delta
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let cred_rev_id =
            if let (Some(rev_reg_id), Some(str_rev_reg), Some((_, _, _, rev_reg_info))) =
                (rev_reg_id, &str_rev_reg, revocation_config_parts)
            {
                let cred_rev_id = rev_reg_info.curr_id.to_string();
                let str_rev_reg_info = serde_json::to_string(&rev_reg_info)?;

                wallet
                    .update_record_value(CATEGORY_REV_REG, &rev_reg_id, str_rev_reg)
                    .await?;

                wallet
                    .update_record_value(CATEGORY_REV_REG_INFO, &rev_reg_id, &str_rev_reg_info)
                    .await?;

                Some(cred_rev_id)
            } else {
                None
            };

        let str_cred = serde_json::to_string(&cred)?;

        Ok((str_cred, cred_rev_id, str_rev_reg_delta))
    }

    /// * `requested_credentials_json`: either a credential or self-attested attribute for each
    ///   requested attribute { "self_attested_attributes": { "self_attested_attribute_referent":
    ///   string }, "requested_attributes": { "requested_attribute_referent_1": {"cred_id": string,
    ///   "timestamp": Optional<number>, revealed: <bool> }}, "requested_attribute_referent_2":
    ///   {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }} },
    ///   "requested_predicates": { "requested_predicates_referent_1": {"cred_id": string,
    ///   "timestamp": Optional<number> }}, } }
    async fn prover_create_proof(
        &self,
        wallet: &impl BaseWallet,
        proof_req_json: &str,
        requested_credentials_json: &str,
        link_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        let pres_req: PresentationRequest = serde_json::from_str(proof_req_json)?;

        let requested_credentials: Value = serde_json::from_str(requested_credentials_json)?;
        let requested_attributes = (&requested_credentials).try_get("requested_attributes")?;

        let requested_predicates = (&requested_credentials).try_get("requested_predicates")?;
        let self_attested_attributes = requested_credentials.get("self_attested_attributes");

        let rev_states: Option<Value> = if let Some(revoc_states_json) = revoc_states_json {
            Some(serde_json::from_str(revoc_states_json)?)
        } else {
            None
        };

        let schemas_: HashMap<SchemaId, Schema> = serde_json::from_str(schemas_json)?;
        let mut schemas: HashMap<SchemaId, CredxSchema> = HashMap::new();
        for (key, value) in schemas_ {
            schemas.insert(key, value.convert(())?);
        }

        let cred_defs: HashMap<CredentialDefinitionId, CredentialDefinition> =
            serde_json::from_str(credential_defs_json)?;

        let mut present_credentials: PresentCredentials = PresentCredentials::new();

        let mut proof_details_by_cred_id: HashMap<
            String,
            (
                CredxCredential,
                Option<u64>,
                Option<CredentialRevocationState>,
                Vec<(String, bool)>,
                Vec<String>,
            ),
        > = HashMap::new();

        // add cred data and referent details for each requested attribute
        for (reft, detail) in requested_attributes.try_as_object()?.iter() {
            let _cred_id = detail.try_get("cred_id")?;
            let cred_id = _cred_id.try_as_str()?;

            let revealed = detail.try_get("revealed")?.try_as_bool()?;

            if let Some((_, _, _, req_attr_refts_revealed, _)) =
                proof_details_by_cred_id.get_mut(cred_id)
            {
                // mapping made for this credential already, add reft and its revealed status
                req_attr_refts_revealed.push((reft.to_string(), revealed));
            } else {
                let credential = Self::_get_credential(wallet, cred_id).await?;

                let (timestamp, rev_state) =
                    get_rev_state(cred_id, &credential, detail, rev_states.as_ref())?;

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
        for (reft, detail) in requested_predicates.try_as_object()?.iter() {
            let _cred_id = detail.try_get("cred_id")?;
            let cred_id = _cred_id.try_as_str()?;

            if let Some((_, _, _, _, req_preds_refts)) = proof_details_by_cred_id.get_mut(cred_id) {
                // mapping made for this credential already, add reft
                req_preds_refts.push(reft.to_string());
            } else {
                let credential = Self::_get_credential(wallet, cred_id).await?;

                let (timestamp, rev_state) =
                    get_rev_state(cred_id, &credential, detail, rev_states.as_ref())?;

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
            let self_attested_obj = self_attested_value.try_as_object()?.clone();
            let self_attested_iter = self_attested_obj.iter();
            for (k, v) in self_attested_iter {
                self_attested_map.insert(k.to_string(), v.try_as_str()?.to_string());
            }

            if self_attested_map.is_empty() {
                None
            } else {
                Some(self_attested_map)
            }
        } else {
            None
        };

        let link_secret = Self::get_link_secret(wallet, link_secret_id).await?;

        let presentation = credx::prover::create_presentation(
            &pres_req,
            present_credentials,
            self_attested,
            &link_secret,
            &hashmap_as_ref(&schemas),
            &hashmap_as_ref(&cred_defs),
        )?;

        Ok(serde_json::to_string(&presentation)?)
    }

    async fn prover_get_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<String> {
        let cred = Self::_get_credential(wallet, cred_id).await?;

        let cred_info = _make_cred_info(cred_id, &cred)?;

        Ok(serde_json::to_string(&cred_info)?)
    }

    async fn prover_get_credentials(
        &self,
        wallet: &impl BaseWallet,
        filter_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        // filter_json should map to WQL query directly
        // TODO - future - may wish to validate the filter_json for more accurate error reporting

        let creds_wql = filter_json.map_or("{}", |x| x);
        let creds = Self::_get_credentials(wallet, creds_wql).await?;

        let cred_info_list: VcxCoreResult<Vec<Value>> = creds
            .iter()
            .map(|(credential_id, cred)| _make_cred_info(credential_id, cred))
            .collect();

        let cred_info_list = cred_info_list?;

        Ok(serde_json::to_string(&cred_info_list)?)
    }

    async fn prover_get_credentials_for_proof_req(
        &self,
        wallet: &impl BaseWallet,
        proof_req: &str,
    ) -> VcxCoreResult<String> {
        let proof_req_v: Value = serde_json::from_str(proof_req).map_err(|e| {
            AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidProofRequest, e)
        })?;

        let requested_attributes = proof_req_v.get("requested_attributes");
        let requested_attributes = if let Some(requested_attributes) = requested_attributes {
            Some(requested_attributes.try_as_object()?.clone())
        } else {
            None
        };
        let requested_predicates = proof_req_v.get("requested_predicates");
        let requested_predicates = if let Some(requested_predicates) = requested_predicates {
            Some(requested_predicates.try_as_object()?.clone())
        } else {
            None
        };

        // handle special case of "empty because json is bad" vs "empty because no attributes
        // sepected"
        if requested_attributes.is_none() && requested_predicates.is_none() {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidAttributesStructure,
                "Invalid Json Parsing of Requested Attributes Retrieved From Libindy",
            ));
        }

        let mut referents: HashSet<String> = HashSet::new();
        if let Some(requested_attributes) = &requested_attributes {
            requested_attributes.iter().for_each(|(k, _)| {
                referents.insert(k.to_string());
            })
        };
        if let Some(requested_predicates) = &requested_predicates {
            requested_predicates.iter().for_each(|(k, _)| {
                referents.insert(k.to_string());
            });
        }

        let mut cred_by_attr: Value = json!({});

        for reft in referents {
            let requested_val = requested_attributes
                .as_ref()
                .and_then(|req_attrs| req_attrs.get(&reft))
                .or_else(|| {
                    requested_predicates
                        .as_ref()
                        .and_then(|req_preds| req_preds.get(&reft))
                })
                .ok_or(AriesVcxCoreError::from_msg(
                    // should not happen
                    AriesVcxCoreErrorKind::InvalidState,
                    format!("Unknown referent: {}", reft),
                ))?;

            let name = requested_val.get("name");
            let names = requested_val.get("names").and_then(|v| v.as_array());

            let attr_names = match (name, names) {
                (Some(name), None) => vec![_normalize_attr_name(name.try_as_str()?)],
                (None, Some(names)) => names
                    .iter()
                    .map(|v| v.try_as_str().map(_normalize_attr_name))
                    .collect::<Result<_, _>>()?,
                _ => Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidInput,
                    "exactly one of 'name' or 'names' must be present",
                ))?,
            };

            let non_revoked = requested_val.get("non_revoked"); // note that aca-py askar fetches from proof_req json
            let restrictions = requested_val.get("restrictions");

            let credx_creds = self
                ._get_credentials_for_proof_req_for_attr_name(wallet, restrictions, attr_names)
                .await?;

            let mut credentials_json = vec![];

            for (cred_id, credx_cred) in credx_creds {
                credentials_json.push(json!({
                    "cred_info": _make_cred_info(&cred_id, &credx_cred)?,
                    "interval": non_revoked
                }))
            }

            cred_by_attr[ATTRS][reft] = Value::Array(credentials_json);
        }

        Ok(serde_json::to_string(&cred_by_attr)?)
    }

    async fn prover_create_credential_req(
        &self,
        wallet: &impl BaseWallet,
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
        link_secret_id: &str,
    ) -> VcxCoreResult<(String, String)> {
        let prover_did = DidValue::new(prover_did, None);
        let cred_def: CredentialDefinition = serde_json::from_str(credential_def_json)?;
        let credential_offer: CredentialOffer = serde_json::from_str(credential_offer_json)?;
        let link_secret = Self::get_link_secret(wallet, link_secret_id).await?;

        let (cred_req, cred_req_metadata) = credx::prover::create_credential_request(
            &prover_did,
            &cred_def,
            &link_secret,
            link_secret_id,
            &credential_offer,
        )?;

        Ok((
            serde_json::to_string(&cred_req)?,
            serde_json::to_string(&cred_req_metadata)?,
        ))
    }

    async fn create_revocation_state(
        &self,
        tails_dir: &str,
        rev_reg_def_json: &str,
        rev_reg_delta_json: &str,
        timestamp: u64,
        cred_rev_id: &str,
    ) -> VcxCoreResult<String> {
        let revoc_reg_def: RevocationRegistryDefinition = serde_json::from_str(rev_reg_def_json)?;
        let tails_file_hash = match revoc_reg_def.borrow() {
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
        let rev_reg_delta: RevocationRegistryDelta = serde_json::from_str(rev_reg_delta_json)?;
        let rev_reg_idx: u32 = cred_rev_id
            .parse()
            .map_err(|e| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ParsingError, e))?;

        let rev_state = credx::prover::create_or_update_revocation_state(
            tails_reader,
            &revoc_reg_def,
            &rev_reg_delta,
            rev_reg_idx,
            timestamp,
            None,
        )?;

        Ok(serde_json::to_string(&rev_state)?)
    }

    async fn prover_store_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxCoreResult<String> {
        let mut credential: CredxCredential = serde_json::from_str(cred_json)?;
        let cred_request_metadata: CredentialRequestMetadata = serde_json::from_str(cred_req_meta)?;
        let link_secret_id = &cred_request_metadata.master_secret_name;
        let link_secret = Self::get_link_secret(wallet, link_secret_id).await?;
        let cred_def: CredentialDefinition = serde_json::from_str(cred_def_json)?;
        let rev_reg_def: Option<RevocationRegistryDefinition> =
            if let Some(rev_reg_def_json) = rev_reg_def_json {
                serde_json::from_str(rev_reg_def_json)?
            } else {
                None
            };

        credx::prover::process_credential(
            &mut credential,
            &cred_request_metadata,
            &link_secret,
            &cred_def,
            rev_reg_def.as_ref(),
        )?;

        let schema_id = &credential.schema_id;
        let (_schema_method, schema_issuer_did, schema_name, schema_version) =
            schema_id.parts().ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidSchema,
                "Could not process credential.schema_id as parts.",
            ))?;

        let cred_def_id = &credential.cred_def_id;
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

        if let Some(rev_reg_id) = &credential.rev_reg_id {
            tags["rev_reg_id"] = serde_json::Value::String(rev_reg_id.0.to_string())
        }

        for (raw_attr_name, attr_value) in credential.values.0.iter() {
            let attr_name = _normalize_attr_name(raw_attr_name);
            // add attribute name and raw value pair
            let value_tag_name = _format_attribute_as_value_tag_name(&attr_name);
            tags[value_tag_name] = Value::String(attr_value.raw.to_string());

            // add attribute name and marker (used for checking existent)
            let marker_tag_name = _format_attribute_as_marker_tag_name(&attr_name);
            tags[marker_tag_name] = Value::String("1".to_string());
        }

        let credential_id = cred_id.map_or(Uuid::new_v4().to_string(), String::from);

        let record_value = serde_json::to_string(&credential)?;
        let tags = serde_json::from_value(tags.clone())?;

        let record = Record::builder()
            .name(credential_id.clone())
            .category(CATEGORY_CREDENTIAL.into())
            .value(record_value)
            .tags(tags)
            .build();

        wallet.add_record(record).await?;

        Ok(credential_id)
    }

    async fn prover_create_link_secret(
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<String> {
        let existing_record = wallet
            .get_record(CATEGORY_LINK_SECRET, link_secret_id)
            .await
            .ok(); // ignore error, as we only care about whether it exists or not

        if existing_record.is_some() {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::DuplicationMasterSecret,
                format!(
                    "Master secret id: {} already exists in wallet.",
                    link_secret_id
                ),
            ));
        }

        let secret = credx::prover::create_link_secret()?;
        let ms_decimal = secret
            .value
            .value()
            .map_err(|err| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::UrsaError,
                    format!(
                        "failed to get BigNumber from master secret, UrsaErrorKind: {:?}",
                        err.kind()
                    ),
                )
            })?
            .to_dec()
            .map_err(|err| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::UrsaError,
                    format!(
                        "Failed convert BigNumber to decimal string, UrsaErrorKind: {:?}",
                        err.kind()
                    ),
                )
            })?;

        let record = Record::builder()
            .name(link_secret_id.into())
            .category(CATEGORY_LINK_SECRET.into())
            .value(ms_decimal)
            .build();

        wallet.add_record(record).await?;

        return Ok(link_secret_id.to_string());
    }

    async fn prover_delete_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<()> {
        wallet.delete_record(CATEGORY_CREDENTIAL, cred_id).await
    }

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxCoreResult<(String, String)> {
        let origin_did = DidValue::new(issuer_did, None);
        let attr_names = serde_json::from_str(attrs)?;

        let schema = credx::issuer::create_schema(&origin_did, name, version, attr_names, None)?;

        let schema_json = serde_json::to_string(&schema)?;
        let schema_id = &schema.id().0;

        Ok((schema_id.to_string(), schema_json))
    }

    async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        _tails_dir: &str,
        rev_reg_id: &str,
        cred_rev_id: &str,
        _rev_reg_delta_json: &str,
    ) -> VcxCoreResult<()> {
        let cred_rev_id = cred_rev_id.parse().map_err(|e| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidInput,
                format!("Invalid cred_rev_id {cred_rev_id} - {e}"),
            )
        })?;

        let rev_reg = Self::get_wallet_record_value(wallet, CATEGORY_REV_REG, rev_reg_id).await?;

        let rev_reg_def =
            Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF, rev_reg_id).await?;

        let rev_reg_priv =
            Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF_PRIV, rev_reg_id).await?;

        let mut rev_reg_info: RevocationRegistryInfo =
            Self::get_wallet_record_value(wallet, CATEGORY_REV_REG_INFO, rev_reg_id).await?;

        let (issuance_type, cred_def_id) = match &rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => {
                (r.value.issuance_type, r.cred_def_id.0.as_str())
            }
        };

        let cred_def =
            Self::get_wallet_record_value(wallet, CATEGORY_CRED_DEF, cred_def_id).await?;

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

        let str_rev_reg_info = serde_json::to_string(&rev_reg_info)?;

        let (rev_reg, new_rev_reg_delta) = credx::issuer::revoke_credential(
            &cred_def,
            &rev_reg_def,
            &rev_reg_priv,
            &rev_reg,
            cred_rev_id,
        )?;

        let old_str_rev_reg_delta = self.get_rev_reg_delta(wallet, rev_reg_id).await?;

        let rev_reg_delta = old_str_rev_reg_delta
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()?;

        let rev_reg_delta = rev_reg_delta
            .map(|rev_reg_delta| {
                credx::issuer::merge_revocation_registry_deltas(&rev_reg_delta, &new_rev_reg_delta)
            })
            .transpose()?
            .unwrap_or(new_rev_reg_delta);

        let str_rev_reg = serde_json::to_string(&rev_reg)?;
        let str_rev_reg_delta = serde_json::to_string(&rev_reg_delta)?;

        wallet
            .update_record_value(CATEGORY_REV_REG, rev_reg_id, &str_rev_reg)
            .await?;

        wallet
            .update_record_value(CATEGORY_REV_REG_INFO, rev_reg_id, &str_rev_reg_info)
            .await?;

        match old_str_rev_reg_delta {
            Some(_) => {
                wallet
                    .update_record_value(CATEGORY_REV_REG_DELTA, rev_reg_id, &str_rev_reg_delta)
                    .await?
            }
            None => {
                let record = Record::builder()
                    .name(rev_reg_id.into())
                    .category(CATEGORY_REV_REG_DELTA.into())
                    .value(str_rev_reg_delta)
                    .build();
                wallet.add_record(record).await?
            }
        }

        Ok(())
    }

    async fn get_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &str,
    ) -> VcxCoreResult<Option<String>> {
        let res_rev_reg_delta = Self::get_wallet_record_value::<RevocationRegistryDelta>(
            wallet,
            CATEGORY_REV_REG_DELTA,
            rev_reg_id,
        )
        .await;

        if let Err(err) = &res_rev_reg_delta {
            warn!(
                "get_rev_reg_delta >> Unable to get rev_reg_delta cache for rev_reg_id: {}, \
                 error: {}",
                rev_reg_id, err
            );
        }

        let res_rev_reg_delta = res_rev_reg_delta
            .ok()
            .as_ref()
            .map(serde_json::to_string)
            .transpose();

        if let Err(err) = &res_rev_reg_delta {
            warn!(
                "get_rev_reg_delta >> Unable to deserialize rev_reg_delta cache for rev_reg_id: \
                 {}, error: {}",
                rev_reg_id, err
            );
        }

        Ok(res_rev_reg_delta.ok().flatten())
    }

    async fn clear_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &str,
    ) -> VcxCoreResult<()> {
        if self.get_rev_reg_delta(wallet, rev_reg_id).await?.is_some() {
            wallet
                .delete_record(CATEGORY_REV_REG_DELTA, rev_reg_id)
                .await?;
        }

        Ok(())
    }

    async fn generate_nonce(&self) -> VcxCoreResult<String> {
        let nonce = credx::verifier::generate_nonce()?.to_string();
        Ok(nonce)
    }
}

fn get_rev_state(
    cred_id: &str,
    credential: &CredxCredential,
    detail: &Value,
    rev_states: Option<&Value>,
) -> VcxCoreResult<(Option<u64>, Option<CredentialRevocationState>)> {
    let timestamp = detail
        .get("timestamp")
        .and_then(|timestamp| timestamp.as_u64());
    let cred_rev_reg_id = credential.rev_reg_id.as_ref().map(|id| id.0.to_string());
    let rev_state = if let (Some(timestamp), Some(cred_rev_reg_id)) = (timestamp, cred_rev_reg_id) {
        let rev_state = rev_states
            .as_ref()
            .and_then(|_rev_states| _rev_states.get(cred_rev_reg_id.to_string()));
        let rev_state = rev_state.ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!(
                "No revocation states provided for credential '{}' with rev_reg_id '{}'",
                cred_id, cred_rev_reg_id
            ),
        ))?;

        let rev_state = rev_state
            .get(timestamp.to_string())
            .ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidJson,
                format!(
                    "No revocation states provided for credential '{}' with rev_reg_id '{}' at \
                     timestamp '{}'",
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

fn _make_cred_info(credential_id: &str, cred: &CredxCredential) -> VcxCoreResult<Value> {
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

fn _format_attribute_as_value_tag_name(attribute_name: &str) -> String {
    format!("attr::{attribute_name}::value")
}

fn _format_attribute_as_marker_tag_name(attribute_name: &str) -> String {
    format!("attr::{attribute_name}::marker")
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
