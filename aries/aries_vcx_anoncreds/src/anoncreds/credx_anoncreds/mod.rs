mod type_conversion;

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};

use anoncreds_types::data_types::{
    identifiers::{
        cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
        schema_id::SchemaId,
    },
    ledger::{
        cred_def::CredentialDefinition,
        rev_reg::RevocationRegistry,
        rev_reg_def::RevocationRegistryDefinition,
        rev_reg_delta::RevocationRegistryDelta,
        schema::{AttributeNames, Schema},
    },
    messages::{
        cred_definition_config::CredentialDefinitionConfig,
        cred_offer::CredentialOffer,
        cred_request::{CredentialRequest, CredentialRequestMetadata},
        cred_selection::{RetrievedCredentialInfo, RetrievedCredentials},
        credential::{Credential, CredentialValues},
        nonce::Nonce,
        pres_request::PresentationRequest,
        presentation::{Presentation, RequestedCredentials},
        revocation_state::CredentialRevocationState,
    },
};
use aries_vcx_wallet::{
    errors::error::VcxWalletResult,
    wallet::{
        base_wallet::{
            record::{AllRecords, Record},
            record_category::RecordCategory,
            record_wallet::RecordWallet,
            BaseWallet,
        },
        record_tags::{RecordTag, RecordTags},
    },
};
use async_trait::async_trait;
use credx::{
    anoncreds_clsignatures::{bn::BigNumber, LinkSecret as ClLinkSecret},
    tails::{TailsFileReader, TailsFileWriter},
    types::{
        Credential as CredxCredential, CredentialDefinition as CredxCredentialDefinition,
        CredentialDefinitionId as CredxCredentialDefinitionId,
        CredentialOffer as CredxCredentialOffer, CredentialRequest as CredxCredentialRequest,
        CredentialRequestMetadata as CredxCredentialRequestMetadata, CredentialRevocationConfig,
        CredentialRevocationState as CredxCredentialRevocationState,
        CredentialValues as CredxCredentialValues, IssuanceType, LinkSecret, PresentCredentials,
        Presentation as CredxPresentation, PresentationRequest as CredxPresentationRequest,
        RegistryType, RevocationRegistry as CredxRevocationRegistry,
        RevocationRegistryDefinition as CredxRevocationRegistryDefinition,
        RevocationRegistryDelta as CredxRevocationRegistryDelta,
        RevocationRegistryId as CredxRevocationRegistryId, Schema as CredxSchema,
        SchemaId as CredxSchemaId,
    },
};
use did_parser_nom::Did;
use indy_credx as credx;
use log::warn;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use type_conversion::Convert;
use uuid::Uuid;

use super::base_anoncreds::{
    BaseAnonCreds, CredentialDefinitionsMap, CredentialId, LinkSecretId, RevocationRegistriesMap,
    RevocationRegistryDefinitionsMap, RevocationStatesMap, SchemasMap,
};
use crate::{
    errors::error::{VcxAnoncredsError, VcxAnoncredsResult},
    utils::{constants::ATTRS, json::AsTypeOrDeserializationError},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: CredxRevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}

/// Adapter used so that credx does not depend strictly on the vdrtools-wallet
/// Will get removed when the wallet and anoncreds interfaces are de-coupled.
#[derive(Debug)]
struct WalletAdapter(Arc<dyn BaseWallet>);

#[async_trait]
#[allow(dead_code)]
impl RecordWallet for WalletAdapter {
    async fn all_records(&self) -> VcxWalletResult<Box<dyn AllRecords + Send>> {
        self.0.all_records().await
    }

    async fn add_record(&self, record: Record) -> VcxWalletResult<()> {
        self.0.add_record(record).await
    }

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<Record> {
        self.0.get_record(category, name).await
    }

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxWalletResult<()> {
        self.0.update_record_tags(category, name, new_tags).await
    }

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxWalletResult<()> {
        self.0.update_record_value(category, name, new_value).await
    }

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<()> {
        self.0.delete_record(category, name).await
    }

    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<String>,
    ) -> VcxWalletResult<Vec<Record>> {
        self.0.search_record(category, search_filter).await
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IndyCredxAnonCreds;

impl IndyCredxAnonCreds {
    async fn get_wallet_record_value<T>(
        wallet: &impl BaseWallet,
        category: RecordCategory,
        id: &str,
    ) -> VcxAnoncredsResult<T>
    where
        T: DeserializeOwned,
    {
        let str_record = wallet.get_record(category, id).await?;
        Ok(serde_json::from_str(str_record.value())?)
    }

    async fn get_link_secret(
        wallet: &impl BaseWallet,
        link_secret_id: &LinkSecretId,
    ) -> VcxAnoncredsResult<LinkSecret> {
        let record = wallet
            .get_record(RecordCategory::LinkSecret, link_secret_id)
            .await?;

        let ms_bn: BigNumber = BigNumber::from_dec(record.value()).map_err(|err| {
            VcxAnoncredsError::UrsaError(format!(
                "Failed to create BigNumber, UrsaErrorKind: {:?}",
                err.kind()
            ))
        })?;
        let ursa_ms: ClLinkSecret = serde_json::from_value(json!({ "ms": ms_bn }))?;

        Ok(LinkSecret { value: ursa_ms })
    }

    async fn _get_credential(
        wallet: &impl BaseWallet,
        credential_id: &str,
    ) -> VcxAnoncredsResult<CredxCredential> {
        let cred_record = wallet
            .get_record(RecordCategory::Cred, credential_id)
            .await?;

        let credential: CredxCredential = serde_json::from_str(cred_record.value())?;

        Ok(credential)
    }

    async fn _get_credentials(
        wallet: &impl BaseWallet,
        wql: &str,
    ) -> VcxAnoncredsResult<Vec<(String, CredxCredential)>> {
        let records = wallet
            .search_record(RecordCategory::Cred, Some(wql.into()))
            .await?;

        let id_cred_tuple_list: VcxAnoncredsResult<Vec<(String, CredxCredential)>> = records
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
    ) -> VcxAnoncredsResult<Vec<(String, CredxCredential)>> {
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
                Value::Null => {
                    json!({ "$and": attrs })
                }
                _ => Err(VcxAnoncredsError::InvalidInput(
                    "Invalid attribute restrictions (must be array or an object)".into(),
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
        proof_req_json: PresentationRequest,
        proof_json: Presentation,
        schemas_json: SchemasMap,
        credential_defs_json: CredentialDefinitionsMap,
        rev_reg_defs_json: Option<RevocationRegistryDefinitionsMap>,
        rev_regs_json: Option<RevocationRegistriesMap>,
    ) -> VcxAnoncredsResult<bool> {
        let presentation: CredxPresentation = proof_json.convert(())?;
        let pres_req: CredxPresentationRequest = proof_req_json.convert(())?;

        let schemas: HashMap<CredxSchemaId, CredxSchema> = schemas_json.convert(())?;

        let cred_defs: HashMap<CredxCredentialDefinitionId, CredxCredentialDefinition> =
            credential_defs_json.convert(())?;

        let rev_reg_defs: Option<
            HashMap<CredxRevocationRegistryId, CredxRevocationRegistryDefinition>,
        > = rev_reg_defs_json.map(|v| v.convert(())).transpose()?;

        let rev_regs: Option<
            HashMap<CredxRevocationRegistryId, HashMap<u64, CredxRevocationRegistry>>,
        > = rev_regs_json.map(|v| v.convert(())).transpose()?;
        let rev_regs: Option<
            HashMap<CredxRevocationRegistryId, HashMap<u64, &CredxRevocationRegistry>>,
        > = rev_regs.as_ref().map(|regs| {
            let mut new_regs: HashMap<
                CredxRevocationRegistryId,
                HashMap<u64, &CredxRevocationRegistry>,
            > = HashMap::new();
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
        issuer_did: &Did,
        cred_def_id: &CredentialDefinitionId,
        tails_dir: &Path,
        max_creds: u32,
        tag: &str,
    ) -> VcxAnoncredsResult<(
        RevocationRegistryDefinitionId,
        RevocationRegistryDefinition,
        RevocationRegistry,
    )> {
        let issuer_did = issuer_did.convert(())?;

        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_str().unwrap().to_string()));

        let cred_def =
            Self::get_wallet_record_value(wallet, RecordCategory::CredDef, &cred_def_id.0).await?;

        let rev_reg_id = credx::issuer::make_revocation_registry_id(
            &issuer_did,
            &cred_def,
            tag,
            RegistryType::CL_ACCUM,
        )?;

        let res_rev_reg =
            Self::get_wallet_record_value(wallet, RecordCategory::RevReg, &rev_reg_id.0).await;
        let res_rev_reg_def =
            Self::get_wallet_record_value(wallet, RecordCategory::RevRegDef, &rev_reg_id.0).await;

        if let (Ok(rev_reg), Ok(rev_reg_def)) = (res_rev_reg, res_rev_reg_def) {
            return Ok((rev_reg_id.to_string().try_into()?, rev_reg, rev_reg_def));
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
            .category(RecordCategory::RevRegInfo)
            .value(str_rev_reg_info)
            .build();
        wallet.add_record(record).await?;

        let str_rev_reg_def = serde_json::to_string(&rev_reg_def)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(RecordCategory::RevRegDef)
            .value(str_rev_reg_def.clone())
            .build();
        wallet.add_record(record).await?;

        let str_rev_reg_def_priv = serde_json::to_string(&rev_reg_def_priv)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(RecordCategory::RevRegDefPriv)
            .value(str_rev_reg_def_priv)
            .build();
        wallet.add_record(record).await?;

        let str_rev_reg = serde_json::to_string(&rev_reg)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(RecordCategory::RevReg)
            .value(str_rev_reg.clone())
            .build();
        wallet.add_record(record).await?;

        Ok((
            rev_reg_id.to_string().try_into()?,
            rev_reg_def.convert(())?,
            rev_reg.convert(())?,
        ))
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &Did,
        _schema_id: &SchemaId,
        schema_json: Schema,
        config_json: CredentialDefinitionConfig,
    ) -> VcxAnoncredsResult<CredentialDefinition> {
        let issuer_did = issuer_did.to_owned();

        let CredentialDefinitionConfig {
            signature_type,
            tag,
            ..
        } = config_json.clone();

        let schema_seq_no = schema_json.seq_no;
        let schema = schema_json.clone().convert(())?;

        let cred_def_id = credx::issuer::make_credential_definition_id(
            &issuer_did.convert(())?,
            schema.id(),
            schema_seq_no,
            &tag,
            signature_type.convert(())?,
        )?;

        // If cred def already exists, return it
        if let Ok(cred_def) =
            Self::get_wallet_record_value(wallet, RecordCategory::CredDef, &cred_def_id.0).await
        {
            // TODO: Perform conversion
            return Ok(cred_def);
        }

        // Otherwise, create cred def
        let (cred_def, cred_def_priv, cred_key_correctness_proof) =
            credx::issuer::create_credential_definition(
                &issuer_did.convert(())?,
                &schema,
                &tag,
                signature_type.convert(())?,
                config_json.convert(())?,
            )?;

        let str_cred_def = serde_json::to_string(&cred_def)?;
        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(RecordCategory::CredDef)
            .value(str_cred_def.clone())
            .build();
        wallet.add_record(record).await?;

        let str_cred_def_priv = serde_json::to_string(&cred_def_priv)?;
        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(RecordCategory::CredDefPriv)
            .value(str_cred_def_priv)
            .build();
        wallet.add_record(record).await?;

        let str_cred_key_proof = serde_json::to_string(&cred_key_correctness_proof)?;
        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(RecordCategory::CredKeyCorrectnessProof)
            .value(str_cred_key_proof)
            .build();
        wallet.add_record(record).await?;

        let record = Record::builder()
            .name(schema.id().to_string())
            .category(RecordCategory::CredSchema)
            .value(serde_json::to_string(&schema_json)?)
            .build();
        let store_schema_res = wallet.add_record(record).await;

        if let Err(e) = store_schema_res {
            warn!(
                "Storing schema {schema_json:?} failed - {e}. It's possible it is already stored."
            )
        }

        let record = Record::builder()
            .name(cred_def_id.0.clone())
            .category(RecordCategory::CredMapSchemaId)
            .value(schema.id().0.clone())
            .build();
        wallet.add_record(record).await?;

        Ok(cred_def.convert((issuer_did.to_string(),))?)
    }

    async fn issuer_create_credential_offer(
        &self,
        wallet: &impl BaseWallet,
        cred_def_id: &CredentialDefinitionId,
    ) -> VcxAnoncredsResult<CredentialOffer> {
        let cred_def =
            Self::get_wallet_record_value(wallet, RecordCategory::CredDef, &cred_def_id.0).await?;

        let correctness_proof = Self::get_wallet_record_value(
            wallet,
            RecordCategory::CredKeyCorrectnessProof,
            &cred_def_id.0,
        )
        .await?;

        let schema = wallet
            .get_record(RecordCategory::CredMapSchemaId, &cred_def_id.0)
            .await?;

        let schema_id = CredxSchemaId(schema.value().into());

        // If cred_def contains schema ID, why take it as an argument here...?
        let offer =
            credx::issuer::create_credential_offer(&schema_id, &cred_def, &correctness_proof)?;

        Ok(offer.convert(())?)
    }

    async fn issuer_create_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_offer_json: CredentialOffer,
        cred_req_json: CredentialRequest,
        cred_values_json: CredentialValues,
        rev_reg_id: Option<&RevocationRegistryDefinitionId>,
        tails_dir: Option<&Path>,
    ) -> VcxAnoncredsResult<(Credential, Option<u32>)> {
        let rev_reg_id = rev_reg_id.map(ToString::to_string);
        let cred_offer: CredxCredentialOffer = cred_offer_json.convert(())?;
        let cred_request: CredxCredentialRequest = cred_req_json.convert(())?;
        let cred_values: CredxCredentialValues = cred_values_json.convert(())?;

        // TODO: Might need to qualify with offer method or something - look into how vdrtools does
        // it
        let cred_def_id = &cred_offer.cred_def_id.0;

        let cred_def =
            Self::get_wallet_record_value(wallet, RecordCategory::CredDef, cred_def_id).await?;

        let cred_def_private =
            Self::get_wallet_record_value(wallet, RecordCategory::CredDefPriv, cred_def_id).await?;

        let mut revocation_config_parts = match &rev_reg_id {
            Some(rev_reg_id) => {
                let rev_reg_def =
                    Self::get_wallet_record_value(wallet, RecordCategory::RevRegDef, rev_reg_id)
                        .await?;

                let rev_reg_def_priv = Self::get_wallet_record_value(
                    wallet,
                    RecordCategory::RevRegDefPriv,
                    rev_reg_id,
                )
                .await?;

                let rev_reg =
                    Self::get_wallet_record_value(wallet, RecordCategory::RevReg, rev_reg_id)
                        .await?;
                let rev_reg_info: RevocationRegistryInfo =
                    Self::get_wallet_record_value(wallet, RecordCategory::RevRegInfo, rev_reg_id)
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

                let CredxRevocationRegistryDefinition::RevocationRegistryDefinitionV1(
                    rev_reg_def_v1,
                ) = rev_reg_def;

                if rev_reg_info.curr_id > rev_reg_def_v1.value.max_cred_num {
                    return Err(VcxAnoncredsError::ActionNotSupported(
                        "The revocation registry is full".into(),
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

        let (cred, rev_reg, _rev_reg_delta) = credx::issuer::create_credential(
            &cred_def,
            &cred_def_private,
            &cred_offer,
            &cred_request,
            cred_values,
            revocation_config,
        )?;

        let str_rev_reg = rev_reg.as_ref().map(serde_json::to_string).transpose()?;

        let cred_rev_id =
            if let (Some(rev_reg_id), Some(str_rev_reg), Some((_, _, _, rev_reg_info))) =
                (rev_reg_id, &str_rev_reg, revocation_config_parts)
            {
                let cred_rev_id = rev_reg_info.curr_id;
                let str_rev_reg_info = serde_json::to_string(&rev_reg_info)?;

                wallet
                    .update_record_value(RecordCategory::RevReg, &rev_reg_id, str_rev_reg)
                    .await?;

                wallet
                    .update_record_value(RecordCategory::RevRegInfo, &rev_reg_id, &str_rev_reg_info)
                    .await?;

                Some(cred_rev_id)
            } else {
                None
            };

        Ok((cred.convert(())?, cred_rev_id))
    }

    async fn prover_create_proof(
        &self,
        wallet: &impl BaseWallet,
        proof_req_json: PresentationRequest,
        requested_credentials_json: RequestedCredentials,
        link_secret_id: &LinkSecretId,
        schemas_json: SchemasMap,
        credential_defs_json: CredentialDefinitionsMap,
        revoc_states_json: Option<RevocationStatesMap>,
    ) -> VcxAnoncredsResult<Presentation> {
        let pres_req: CredxPresentationRequest = proof_req_json.convert(())?;

        let requested_attributes = requested_credentials_json.requested_attributes;
        let requested_predicates = requested_credentials_json.requested_predicates;
        let self_attested_attributes = requested_credentials_json.self_attested_attributes;

        let schemas: HashMap<CredxSchemaId, CredxSchema> = schemas_json.convert(())?;

        let mut present_credentials: PresentCredentials = PresentCredentials::new();

        let mut proof_details_by_cred_id: HashMap<
            String,
            (
                CredxCredential,
                Option<u64>,
                Option<CredxCredentialRevocationState>,
                Vec<(String, bool)>,
                Vec<String>,
            ),
        > = HashMap::new();

        // add cred data and referent details for each requested attribute
        for (reft, detail) in requested_attributes {
            let cred_id = &detail.cred_id;
            let revealed = detail.revealed;

            if let Some((_, _, _, req_attr_refts_revealed, _)) =
                proof_details_by_cred_id.get_mut(cred_id)
            {
                // mapping made for this credential already, add reft and its revealed status
                req_attr_refts_revealed.push((reft.to_string(), revealed));
            } else {
                let credential = Self::_get_credential(wallet, cred_id).await?;

                let (timestamp, rev_state) = get_rev_state(
                    cred_id,
                    &credential,
                    detail.timestamp,
                    revoc_states_json.as_ref(),
                )?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential,
                        timestamp,
                        rev_state.map(|v| v.convert(())).transpose()?,
                        vec![(reft.to_string(), revealed)],
                        vec![],
                    ),
                );
            }
        }

        // add cred data and referent details for each requested predicate
        for (reft, detail) in requested_predicates {
            let cred_id = &detail.cred_id;

            if let Some((_, _, _, _, req_preds_refts)) = proof_details_by_cred_id.get_mut(cred_id) {
                // mapping made for this credential already, add reft
                req_preds_refts.push(reft.to_string());
            } else {
                let credential = Self::_get_credential(wallet, cred_id).await?;

                let (timestamp, rev_state) = get_rev_state(
                    cred_id,
                    &credential,
                    detail.timestamp,
                    revoc_states_json.as_ref(),
                )?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential,
                        timestamp,
                        rev_state.map(|v| v.convert(())).transpose()?,
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

        let link_secret = Self::get_link_secret(wallet, link_secret_id).await?;

        let presentation = credx::prover::create_presentation(
            &pres_req,
            present_credentials,
            Some(self_attested_attributes),
            &link_secret,
            &hashmap_as_ref(&schemas),
            &hashmap_as_ref(&credential_defs_json.convert(())?),
        )?;

        Ok(presentation.convert(())?)
    }

    async fn prover_get_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &CredentialId,
    ) -> VcxAnoncredsResult<RetrievedCredentialInfo> {
        let cred = Self::_get_credential(wallet, cred_id).await?;

        _make_cred_info(cred_id, &cred)
    }

    async fn prover_get_credentials(
        &self,
        wallet: &impl BaseWallet,
        filter_json: Option<&str>,
    ) -> VcxAnoncredsResult<Vec<RetrievedCredentialInfo>> {
        // filter_json should map to WQL query directly
        // TODO - future - may wish to validate the filter_json for more accurate error reporting

        let creds_wql = filter_json.map_or("{}", |x| x);
        let creds = Self::_get_credentials(wallet, creds_wql).await?;

        creds
            .iter()
            .map(|(credential_id, cred)| _make_cred_info(credential_id, cred))
            .collect()
    }

    async fn prover_get_credentials_for_proof_req(
        &self,
        wallet: &impl BaseWallet,
        proof_req: PresentationRequest,
    ) -> VcxAnoncredsResult<RetrievedCredentials> {
        let proof_req_v: Value = serde_json::to_value(proof_req)
            .map_err(|e| VcxAnoncredsError::InvalidProofRequest(e.to_string()))?;

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
            return Err(VcxAnoncredsError::InvalidAttributesStructure(
                "Invalid Json Parsing of Requested Attributes Retrieved From Libindy".into(),
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
                .ok_or(
                    // should not happen
                    VcxAnoncredsError::InvalidState(format!("Unknown referent: {}", reft)),
                )?;

            let name = requested_val.get("name");
            let names = requested_val.get("names").and_then(|v| v.as_array());

            let attr_names = match (name, names) {
                (Some(name), None) => vec![_normalize_attr_name(name.try_as_str()?)],
                (None, Some(names)) => names
                    .iter()
                    .map(|v| v.try_as_str().map(_normalize_attr_name))
                    .collect::<Result<_, _>>()?,
                _ => Err(VcxAnoncredsError::InvalidInput(
                    "exactly one of 'name' or 'names' must be present".into(),
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

        Ok(serde_json::from_value(cred_by_attr)?)
    }

    async fn prover_create_credential_req(
        &self,
        wallet: &impl BaseWallet,
        prover_did: &Did,
        cred_offer_json: CredentialOffer,
        credential_def_json: CredentialDefinition,
        link_secret_id: &LinkSecretId,
    ) -> VcxAnoncredsResult<(CredentialRequest, CredentialRequestMetadata)> {
        let prover_did = prover_did.convert(())?;
        let cred_def: CredxCredentialDefinition = credential_def_json.convert(())?;
        let credential_offer: CredxCredentialOffer = cred_offer_json.convert(())?;
        let link_secret = Self::get_link_secret(wallet, link_secret_id).await?;

        let (cred_req, cred_req_metadata) = credx::prover::create_credential_request(
            &prover_did,
            &cred_def,
            &link_secret,
            link_secret_id,
            &credential_offer,
        )?;

        Ok((cred_req.convert(())?, cred_req_metadata.convert(())?))
    }

    async fn create_revocation_state(
        &self,
        tails_dir: &Path,
        rev_reg_def_json: RevocationRegistryDefinition,
        rev_reg_delta_json: RevocationRegistryDelta,
        timestamp: u64,
        cred_rev_id: u32,
    ) -> VcxAnoncredsResult<CredentialRevocationState> {
        let revoc_reg_def: CredxRevocationRegistryDefinition = rev_reg_def_json.convert(())?;
        let tails_file_hash = match revoc_reg_def.borrow() {
            CredxRevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => {
                &r.value.tails_hash
            }
        };

        let mut tails_file_path = std::path::PathBuf::new();
        tails_file_path.push(tails_dir);
        tails_file_path.push(tails_file_hash);

        let tails_path = tails_file_path.to_str().ok_or_else(|| {
            VcxAnoncredsError::InvalidOption("tails file is not an unicode string".into())
        })?;

        let tails_reader = TailsFileReader::new(tails_path);
        let rev_reg_delta: CredxRevocationRegistryDelta = rev_reg_delta_json.convert(())?;

        let rev_state = credx::prover::create_or_update_revocation_state(
            tails_reader,
            &revoc_reg_def,
            &rev_reg_delta,
            cred_rev_id,
            timestamp,
            None,
        )?;

        Ok(rev_state.convert(())?)
    }

    async fn prover_store_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_req_meta: CredentialRequestMetadata,
        cred_json: Credential,
        cred_def_json: CredentialDefinition,
        rev_reg_def_json: Option<RevocationRegistryDefinition>,
    ) -> VcxAnoncredsResult<CredentialId> {
        let mut credential: CredxCredential = cred_json.convert(())?;
        let cred_request_metadata: CredxCredentialRequestMetadata = cred_req_meta.convert(())?;
        let link_secret_id = &cred_request_metadata.master_secret_name;
        let link_secret = Self::get_link_secret(wallet, link_secret_id).await?;
        let cred_def: CredxCredentialDefinition = cred_def_json.convert(())?;
        let rev_reg_def: Option<CredxRevocationRegistryDefinition> =
            if let Some(rev_reg_def_json) = rev_reg_def_json {
                Some(rev_reg_def_json.convert(())?)
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
            schema_id.parts().ok_or(VcxAnoncredsError::InvalidSchema(
                "Could not process credential.schema_id as parts.".into(),
            ))?;

        let cred_def_id = &credential.cred_def_id;
        let (_cred_def_method, issuer_did, _signature_type, _schema_id, _tag) =
            cred_def_id.parts().ok_or(VcxAnoncredsError::InvalidSchema(
                "Could not process credential.cred_def_id as parts.".into(),
            ))?;

        let mut tags = RecordTags::new(vec![
            RecordTag::new("schema_id", &schema_id.0),
            RecordTag::new("schema_issuer_did", &schema_issuer_did.0),
            RecordTag::new("schema_name", &schema_name),
            RecordTag::new("schema_version", &schema_version),
            RecordTag::new("issuer_did", &issuer_did.0),
            RecordTag::new("cred_def_id", &cred_def_id.0),
        ]);

        if let Some(rev_reg_id) = &credential.rev_reg_id {
            tags.add(RecordTag::new("rev_reg_id", &rev_reg_id.0));
        }

        for (raw_attr_name, attr_value) in credential.values.0.iter() {
            let attr_name = _normalize_attr_name(raw_attr_name);
            // add attribute name and raw value pair
            let value_tag_name = _format_attribute_as_value_tag_name(&attr_name);
            tags.add(RecordTag::new(&value_tag_name, &attr_value.raw));

            // add attribute name and marker (used for checking existent)
            let marker_tag_name = _format_attribute_as_marker_tag_name(&attr_name);
            tags.add(RecordTag::new(&marker_tag_name, "1"))
        }

        let credential_id = Uuid::new_v4().to_string();

        let record_value = serde_json::to_string(&credential)?;

        let record = Record::builder()
            .name(credential_id.clone())
            .category(RecordCategory::Cred)
            .value(record_value)
            .tags(tags)
            .build();

        wallet.add_record(record).await?;

        Ok(credential_id)
    }

    async fn prover_create_link_secret(
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &LinkSecretId,
    ) -> VcxAnoncredsResult<()> {
        let existing_record = wallet
            .get_record(RecordCategory::LinkSecret, link_secret_id)
            .await
            .ok(); // ignore error, as we only care about whether it exists or not

        if existing_record.is_some() {
            return Err(VcxAnoncredsError::DuplicationMasterSecret(format!(
                "Master secret id: {} already exists in wallet.",
                link_secret_id
            )));
        }

        let secret = credx::prover::create_link_secret()?;
        let ms_decimal = secret
            .value
            .value()
            .map_err(|err| {
                VcxAnoncredsError::UrsaError(format!(
                    "failed to get BigNumber from master secret, UrsaErrorKind: {:?}",
                    err.kind()
                ))
            })?
            .to_dec()
            .map_err(|err| {
                VcxAnoncredsError::UrsaError(format!(
                    "Failed convert BigNumber to decimal string, UrsaErrorKind: {:?}",
                    err.kind()
                ))
            })?;

        let record = Record::builder()
            .name(link_secret_id.into())
            .category(RecordCategory::LinkSecret)
            .value(ms_decimal)
            .build();

        wallet.add_record(record).await?;
        Ok(())
    }

    async fn prover_delete_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &CredentialId,
    ) -> VcxAnoncredsResult<()> {
        Ok(wallet.delete_record(RecordCategory::Cred, cred_id).await?)
    }

    async fn issuer_create_schema(
        &self,
        issuer_did: &Did,
        name: &str,
        version: &str,
        attrs: AttributeNames,
    ) -> VcxAnoncredsResult<Schema> {
        Ok(credx::issuer::create_schema(
            &issuer_did.convert(())?,
            name,
            version,
            attrs.convert(())?,
            None,
        )?
        .convert((issuer_did.to_string(),))?)
    }

    async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        cred_rev_id: u32,
        _rev_reg_delta_json: RevocationRegistryDelta,
    ) -> VcxAnoncredsResult<()> {
        let rev_reg_id_str = &rev_reg_id.to_string();

        let rev_reg =
            Self::get_wallet_record_value(wallet, RecordCategory::RevReg, rev_reg_id_str).await?;

        let rev_reg_def =
            Self::get_wallet_record_value(wallet, RecordCategory::RevRegDef, rev_reg_id_str)
                .await?;

        let rev_reg_priv =
            Self::get_wallet_record_value(wallet, RecordCategory::RevRegDefPriv, rev_reg_id_str)
                .await?;

        let mut rev_reg_info: RevocationRegistryInfo =
            Self::get_wallet_record_value(wallet, RecordCategory::RevRegInfo, rev_reg_id_str)
                .await?;

        let (issuance_type, cred_def_id) = match &rev_reg_def {
            CredxRevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => {
                (r.value.issuance_type, r.cred_def_id.0.as_str())
            }
        };

        let cred_def =
            Self::get_wallet_record_value(wallet, RecordCategory::CredDef, cred_def_id).await?;

        match issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.remove(&cred_rev_id) {
                    return Err(VcxAnoncredsError::InvalidInput(format!(
                        "Revocation id: {:?} not found in RevocationRegistry",
                        cred_rev_id
                    )));
                };
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.insert(cred_rev_id) {
                    return Err(VcxAnoncredsError::InvalidInput(format!(
                        "Revocation id: {:?} not found in RevocationRegistry",
                        cred_rev_id
                    )));
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
            .to_owned()
            .map(|v| v.convert(()))
            .transpose()?
            .map(|rev_reg_delta: CredxRevocationRegistryDelta| {
                credx::issuer::merge_revocation_registry_deltas(&rev_reg_delta, &new_rev_reg_delta)
            })
            .transpose()?
            .unwrap_or(new_rev_reg_delta);

        let str_rev_reg = serde_json::to_string(&rev_reg)?;
        let str_rev_reg_delta = serde_json::to_string(&rev_reg_delta)?;

        wallet
            .update_record_value(RecordCategory::RevReg, rev_reg_id_str, &str_rev_reg)
            .await?;

        wallet
            .update_record_value(
                RecordCategory::RevRegInfo,
                rev_reg_id_str,
                &str_rev_reg_info,
            )
            .await?;

        match old_str_rev_reg_delta {
            Some(_) => {
                wallet
                    .update_record_value(
                        RecordCategory::RevRegDelta,
                        rev_reg_id_str,
                        &str_rev_reg_delta,
                    )
                    .await?
            }
            None => {
                let record = Record::builder()
                    .name(rev_reg_id_str.into())
                    .category(RecordCategory::RevRegDelta)
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
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxAnoncredsResult<Option<RevocationRegistryDelta>> {
        let res_rev_reg_delta = Self::get_wallet_record_value::<RevocationRegistryDelta>(
            wallet,
            RecordCategory::RevRegDelta,
            &rev_reg_id.to_string(),
        )
        .await;

        if let Err(err) = &res_rev_reg_delta {
            warn!(
                "get_rev_reg_delta >> Unable to get rev_reg_delta cache for rev_reg_id: {}, \
                 error: {}",
                rev_reg_id, err
            );
        }

        Ok(res_rev_reg_delta.ok())
    }

    async fn clear_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxAnoncredsResult<()> {
        if self.get_rev_reg_delta(wallet, rev_reg_id).await?.is_some() {
            wallet
                .delete_record(RecordCategory::RevRegDelta, &rev_reg_id.to_string())
                .await?;
        }

        Ok(())
    }

    async fn generate_nonce(&self) -> VcxAnoncredsResult<Nonce> {
        Ok(Nonce::from_dec(credx::verifier::generate_nonce()?.as_ref()).unwrap())
    }
}

fn get_rev_state(
    cred_id: &str,
    credential: &CredxCredential,
    timestamp: Option<u64>,
    rev_states: Option<&RevocationStatesMap>,
) -> VcxAnoncredsResult<(Option<u64>, Option<CredentialRevocationState>)> {
    let cred_rev_reg_id = credential.rev_reg_id.as_ref().map(|id| id.0.to_string());
    let rev_state = if let (Some(timestamp), Some(cred_rev_reg_id)) = (timestamp, cred_rev_reg_id) {
        let rev_state = rev_states
            .as_ref()
            .and_then(|_rev_states| _rev_states.get(&cred_rev_reg_id));
        let rev_state = rev_state.ok_or(VcxAnoncredsError::InvalidJson(format!(
            "No revocation states provided for credential '{}' with rev_reg_id '{}'",
            cred_id, cred_rev_reg_id
        )))?;

        let rev_state = rev_state
            .get(&timestamp)
            .ok_or(VcxAnoncredsError::InvalidJson(format!(
                "No revocation states provided for credential '{}' with rev_reg_id '{}' at \
                 timestamp '{}'",
                cred_id, cred_rev_reg_id, timestamp
            )))?;

        Some(rev_state.clone())
    } else {
        None
    };

    Ok((timestamp, rev_state))
}

fn _normalize_attr_name(name: &str) -> String {
    // "name": string, // attribute name, (case insensitive and ignore spaces)
    name.replace(' ', "").to_lowercase()
}

fn _make_cred_info(
    credential_id: &str,
    cred: &CredxCredential,
) -> VcxAnoncredsResult<RetrievedCredentialInfo> {
    let cred_sig = serde_json::to_value(&cred.signature)?;

    let rev_info = cred_sig.get("r_credential");

    let cred_rev_id: Option<u32> = rev_info
        .and_then(|x| x.get("i"))
        .and_then(|i| i.as_u64().map(|i| i as u32));

    let mut attributes = HashMap::new();
    for (x, y) in cred.values.0.iter() {
        attributes.insert(x.to_string(), y.raw.to_string());
    }

    Ok(RetrievedCredentialInfo {
        referent: credential_id.to_string(),
        attributes,
        schema_id: SchemaId::try_from(cred.schema_id.to_string())?,
        cred_def_id: cred.cred_def_id.clone().convert(())?,
        rev_reg_id: cred.rev_reg_id.as_ref().map(|x| x.0.to_string()),
        cred_rev_id,
    })
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
