mod type_conversion;

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anoncreds::{
    cl::{Accumulator, RevocationRegistry as CryptoRevocationRegistry},
    data_types::{
        cred_def::{
            CredentialDefinition as AnoncredsCredentialDefinition,
            CredentialDefinitionId as AnoncredsCredentialDefinitionId, CL_SIGNATURE_TYPE,
        },
        credential::Credential as AnoncredsCredential,
        issuer_id::IssuerId,
        rev_reg_def::{
            RevocationRegistryDefinitionId as AnoncredsRevocationRegistryDefinitionId, CL_ACCUM,
        },
        schema::{Schema as AnoncredsSchema, SchemaId as AnoncredsSchemaId},
    },
    issuer::{create_revocation_registry_def, create_revocation_status_list},
    tails::TailsFileWriter,
    types::{
        CredentialOffer as AnoncredsCredentialOffer,
        CredentialRequest as AnoncredsCredentialRequest,
        CredentialRequestMetadata as AnoncredsCredentialRequestMetadata,
        CredentialRevocationConfig,
        CredentialRevocationState as AnoncredsCredentialRevocationState,
        CredentialValues as AnoncredsCredentialValues, LinkSecret, PresentCredentials,
        Presentation as AnoncredsPresentation, PresentationRequest as AnoncredsPresentationRequest,
        RegistryType, RevocationRegistry as AnoncredsRevocationRegistry,
        RevocationRegistryDefinition as AnoncredsRevocationRegistryDefinition,
    },
};
use anoncreds_types::{
    data_types::{
        identifiers::{
            cred_def_id::CredentialDefinitionId, rev_reg_def_id::RevocationRegistryDefinitionId,
            schema_id::SchemaId,
        },
        ledger::{
            cred_def::{CredentialDefinition, SignatureType},
            rev_reg::RevocationRegistry,
            rev_reg_def::RevocationRegistryDefinition,
            rev_reg_delta::{RevocationRegistryDelta, RevocationRegistryDeltaValue},
            rev_status_list::RevocationStatusList,
            schema::{AttributeNames, Schema},
        },
        messages::{
            cred_definition_config::CredentialDefinitionConfig,
            cred_offer::CredentialOffer,
            cred_request::{CredentialRequest, CredentialRequestMetadata},
            cred_selection::{
                RetrievedCredentialForReferent, RetrievedCredentialInfo, RetrievedCredentials,
            },
            credential::{Credential, CredentialValues},
            nonce::Nonce,
            pres_request::PresentationRequest,
            presentation::{Presentation, RequestedCredentials},
            revocation_state::CredentialRevocationState,
        },
    },
    utils::conversions::from_revocation_registry_delta_to_revocation_status_list,
};
use aries_vcx_wallet::wallet::{
    base_wallet::{record::Record, record_category::RecordCategory, BaseWallet},
    record_tags::{RecordTag, RecordTags},
};
use async_trait::async_trait;
use did_parser_nom::Did;
use log::warn;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use time::OffsetDateTime;
use uuid::Uuid;

use super::base_anoncreds::{
    BaseAnonCreds, CredentialDefinitionsMap, CredentialId, LinkSecretId, RevocationRegistriesMap,
    RevocationRegistryDefinitionsMap, RevocationStatesMap, SchemasMap,
};
use crate::{
    anoncreds::anoncreds::type_conversion::Convert,
    errors::error::{VcxAnoncredsError, VcxAnoncredsResult},
};

fn from_revocation_status_list_to_revocation_registry_delta(
    rev_status_list: &RevocationStatusList,
    prev_accum: Option<Accumulator>,
) -> VcxAnoncredsResult<RevocationRegistryDelta> {
    let _issued = rev_status_list
        .state()
        .iter()
        .enumerate()
        .filter_map(
            |(idx, is_revoked)| {
                if !is_revoked {
                    Some(idx as u32)
                } else {
                    None
                }
            },
        )
        .collect::<Vec<_>>();
    let revoked = rev_status_list
        .state()
        .iter()
        .enumerate()
        .filter_map(
            |(idx, is_revoked)| {
                if *is_revoked {
                    Some(idx as u32)
                } else {
                    None
                }
            },
        )
        .collect::<Vec<_>>();

    let registry = CryptoRevocationRegistry {
        accum: rev_status_list.accum().ok_or_else(|| {
            VcxAnoncredsError::InvalidState(
                "Revocation registry delta cannot be created from revocation status list without \
                 accumulator"
                    .into(),
            )
        })?,
    };

    Ok(RevocationRegistryDelta {
        value: RevocationRegistryDeltaValue {
            prev_accum,
            accum: registry.accum,
            issued: Default::default(), // TODO
            revoked,
        },
    })
}

#[derive(Debug, Copy, Clone)]
pub struct Anoncreds;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryDefinitionId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}

impl Anoncreds {
    async fn get_wallet_record_value<T>(
        &self,
        wallet: &impl BaseWallet,
        category: RecordCategory,
        id: &str,
    ) -> VcxAnoncredsResult<T>
    where
        T: DeserializeOwned,
    {
        let str_record = wallet.get_record(category, id).await?;
        serde_json::from_str(str_record.value()).map_err(From::from)
    }

    async fn get_link_secret(
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &LinkSecretId,
    ) -> VcxAnoncredsResult<LinkSecret> {
        let ms_decimal = wallet
            .get_record(RecordCategory::LinkSecret, link_secret_id)
            .await?;

        Ok(ms_decimal.value().try_into().unwrap())
    }

    async fn _get_credentials(
        wallet: &impl BaseWallet,
        wql: &str,
    ) -> VcxAnoncredsResult<Vec<(String, Credential)>> {
        let records = wallet
            .search_record(RecordCategory::Cred, Some(wql.into()))
            .await?;

        let id_cred_tuple_list: VcxAnoncredsResult<Vec<(String, Credential)>> = records
            .into_iter()
            .map(|record| {
                let credential: Credential = serde_json::from_str(record.value())?;

                Ok((record.name().into(), credential))
            })
            .collect();

        id_cred_tuple_list
    }

    async fn _get_credentials_for_proof_req_for_attr_name(
        &self,
        wallet: &impl BaseWallet,
        restrictions: Option<Value>,
        attr_names: Vec<String>,
    ) -> VcxAnoncredsResult<Vec<(String, Credential)>> {
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
impl BaseAnonCreds for Anoncreds {
    async fn verifier_verify_proof(
        &self,
        proof_request_json: PresentationRequest,
        proof_json: Presentation,
        schemas_json: SchemasMap,
        credential_defs_json: CredentialDefinitionsMap,
        rev_reg_defs_json: Option<RevocationRegistryDefinitionsMap>,
        rev_regs_json: Option<RevocationRegistriesMap>,
    ) -> VcxAnoncredsResult<bool> {
        let presentation: AnoncredsPresentation = proof_json.convert(())?;
        let pres_req: AnoncredsPresentationRequest = proof_request_json.convert(())?;

        let schemas: HashMap<AnoncredsSchemaId, AnoncredsSchema> = schemas_json.convert(())?;

        let cred_defs: HashMap<AnoncredsCredentialDefinitionId, AnoncredsCredentialDefinition> =
            credential_defs_json.convert(())?;

        // tack on issuerId for ease of processing status lists
        let rev_regs_map_with_issuer_ids: Option<HashMap<_, _>> =
            match (rev_regs_json, &rev_reg_defs_json) {
                (Some(regs), Some(defs)) => Some(
                    regs.into_iter()
                        .filter_map(|(k, v)| {
                            let def = defs.get(&k)?;
                            Some((k, (v, def.issuer_id.clone())))
                        })
                        .collect(),
                ),
                _ => None,
            };

        let rev_reg_defs: Option<
            HashMap<AnoncredsRevocationRegistryDefinitionId, AnoncredsRevocationRegistryDefinition>,
        > = rev_reg_defs_json.map(|v| v.convert(())).transpose()?;

        let rev_status_lists = rev_regs_map_with_issuer_ids
            .map(|r| r.convert(()))
            .transpose()?;

        Ok(anoncreds::verifier::verify_presentation(
            &presentation,
            &pres_req,
            &schemas,
            &cred_defs,
            rev_reg_defs.as_ref(),
            rev_status_lists,
            None, // no idea what this is
        )?)
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
        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_str().unwrap().to_string()));

        let cred_def: AnoncredsCredentialDefinition = self
            .get_wallet_record_value(wallet, RecordCategory::CredDef, &cred_def_id.to_string())
            .await?;
        let rev_reg_id =
            make_revocation_registry_id(issuer_did, cred_def_id, tag, RegistryType::CL_ACCUM)?;

        let (rev_reg_def, rev_reg_def_priv) = create_revocation_registry_def(
            &cred_def,
            AnoncredsCredentialDefinitionId::new(cred_def_id.to_string()).unwrap(),
            tag,
            RegistryType::CL_ACCUM,
            max_creds,
            &mut tails_writer,
        )?;

        let timestamp = OffsetDateTime::now_utc().unix_timestamp() as u64;

        let rev_status_list = create_revocation_status_list(
            &cred_def,
            AnoncredsRevocationRegistryDefinitionId::new(rev_reg_id.to_string()).unwrap(),
            &rev_reg_def,
            &rev_reg_def_priv,
            true,
            Some(timestamp),
        )?;

        let opt_rev_reg: Option<CryptoRevocationRegistry> = (&rev_status_list).into();
        let rev_reg = opt_rev_reg
            .expect("creating a RevocationStatusList always generates a CryptoRevocationRegistry");

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

        let mut rev_reg_def_val = serde_json::to_value(&rev_reg_def)?;
        rev_reg_def_val
            .as_object_mut()
            .unwrap()
            .insert("id".to_owned(), rev_reg_id.0.clone().into());
        rev_reg_def_val
            .as_object_mut()
            .unwrap()
            .insert("ver".to_owned(), "1.0".into());
        rev_reg_def_val["value"]
            .as_object_mut()
            .unwrap()
            .insert("issuanceType".to_string(), "ISSUANCE_BY_DEFAULT".into());

        let str_rev_reg_def = serde_json::to_string(&rev_reg_def_val)?;
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

        let rev_reg = AnoncredsRevocationRegistry { value: rev_reg };
        let str_rev_reg = serde_json::to_string(&rev_reg)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(RecordCategory::RevReg)
            .value(str_rev_reg.clone())
            .build();
        wallet.add_record(record).await?;

        Ok((
            rev_reg_id.to_string().try_into()?,
            rev_reg_def.convert((rev_reg_id.to_string(),))?,
            rev_reg.convert(())?,
        ))
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        wallet: &impl BaseWallet,
        issuer_did: &Did,
        schema_id: &SchemaId,
        schema_json: Schema,
        config_json: CredentialDefinitionConfig,
    ) -> VcxAnoncredsResult<CredentialDefinition> {
        let CredentialDefinitionConfig {
            tag,
            signature_type,
            ..
        } = config_json.clone();

        let cred_def_id = make_credential_definition_id(
            issuer_did,
            schema_id,
            schema_json.seq_no,
            &tag,
            signature_type,
        );

        // If cred def already exists, return it
        if let Ok(cred_def) = self
            .get_wallet_record_value(wallet, RecordCategory::CredDef, &cred_def_id.0)
            .await
        {
            // TODO! Convert?
            return Ok(cred_def);
        }

        // Otherwise, create cred def
        let (cred_def, cred_def_priv, cred_key_correctness_proof) =
            anoncreds::issuer::create_credential_definition(
                // Schema ID must be just the schema seq no for some reason
                AnoncredsSchemaId::new_unchecked(schema_json.seq_no.unwrap().to_string()),
                // SchemaId::new(schema_id).unwrap(),
                &schema_json.clone().convert(())?,
                schema_json.issuer_id.clone().convert(())?,
                &tag,
                signature_type.convert(())?,
                config_json.convert(())?,
            )?;

        let mut cred_def_val = serde_json::to_value(&cred_def)?;
        cred_def_val
            .as_object_mut()
            .map(|v| v.insert("id".to_owned(), cred_def_id.to_string().into()));

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
            .name(schema_id.to_string())
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
            // .value(schema_id.to_string())
            .value(json!({ "schemaId": schema_id }).to_string())
            .build();
        wallet.add_record(record).await?;

        Ok(cred_def.convert((cred_def_id.to_string(),))?)
    }

    async fn issuer_create_credential_offer(
        &self,
        wallet: &impl BaseWallet,
        cred_def_id: &CredentialDefinitionId,
    ) -> VcxAnoncredsResult<CredentialOffer> {
        let correctness_proof = self
            .get_wallet_record_value(
                wallet,
                RecordCategory::CredKeyCorrectnessProof,
                &cred_def_id.to_string(),
            )
            .await?;

        let schema_id_value = self
            .get_wallet_record_value::<Value>(
                wallet,
                RecordCategory::CredMapSchemaId,
                &cred_def_id.to_string(),
            )
            .await?;

        let offer = anoncreds::issuer::create_credential_offer(
            AnoncredsSchemaId::new(schema_id_value["schemaId"].as_str().unwrap()).unwrap(),
            AnoncredsCredentialDefinitionId::new(cred_def_id.to_string()).unwrap(),
            &correctness_proof,
        )?;

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
        let cred_offer: AnoncredsCredentialOffer = cred_offer_json.convert(())?;
        let cred_request: AnoncredsCredentialRequest = cred_req_json.convert(())?;
        let cred_values: AnoncredsCredentialValues = cred_values_json.convert(())?;

        let cred_def_id = &cred_offer.cred_def_id.0;

        let cred_def = self
            .get_wallet_record_value(wallet, RecordCategory::CredDef, cred_def_id)
            .await?;

        let cred_def_private = self
            .get_wallet_record_value(wallet, RecordCategory::CredDefPriv, cred_def_id)
            .await?;

        let rev_reg_id = rev_reg_id.map(ToString::to_string);
        let mut revocation_config_parts = match (tails_dir, &rev_reg_id) {
            (Some(tails_dir), Some(rev_reg_def_id)) => {
                let rev_reg_def: AnoncredsRevocationRegistryDefinition = self
                    .get_wallet_record_value(wallet, RecordCategory::RevRegDef, rev_reg_def_id)
                    .await?;

                let rev_reg_def_priv = self
                    .get_wallet_record_value(wallet, RecordCategory::RevRegDefPriv, rev_reg_def_id)
                    .await?;

                let rev_reg: AnoncredsRevocationRegistry = self
                    .get_wallet_record_value(wallet, RecordCategory::RevReg, rev_reg_def_id)
                    .await?;
                let rev_reg_info: RevocationRegistryInfo = self
                    .get_wallet_record_value(wallet, RecordCategory::RevRegInfo, rev_reg_def_id)
                    .await?;

                let rev_reg_def_id =
                    AnoncredsRevocationRegistryDefinitionId::new(rev_reg_def_id).unwrap();

                Some((
                    rev_reg_def,
                    rev_reg_def_id,
                    rev_reg_def_priv,
                    rev_reg,
                    rev_reg_info,
                    tails_dir,
                ))
            }
            (None, None) => None,
            (tails_dir, rev_reg_def_id) => {
                warn!(
                    "Missing revocation config params: tails_dir: {tails_dir:?} - \
                     {rev_reg_def_id:?}; Issuing non revokable credential"
                );
                None
            }
        };

        let rev_status_list = match &revocation_config_parts {
            Some((rev_reg_def, rev_reg_def_id, rev_reg_def_priv, _, _, _)) => {
                Some(create_revocation_status_list(
                    &cred_def,
                    rev_reg_def_id.clone(),
                    rev_reg_def,
                    rev_reg_def_priv,
                    true,
                    None,
                )?)
            }
            _ => None,
        };

        let revocation_config = match (&mut revocation_config_parts, &rev_status_list) {
            (
                Some((rev_reg_def, _, rev_reg_def_priv, _, rev_reg_info, _)),
                Some(rev_status_list),
            ) => {
                rev_reg_info.curr_id += 1;

                if rev_reg_info.curr_id > rev_reg_def.value.max_cred_num {
                    return Err(VcxAnoncredsError::ActionNotSupported(
                        "The revocation registry is full".into(),
                    ));
                }

                rev_reg_info.used_ids.insert(rev_reg_info.curr_id);

                let revocation_config = CredentialRevocationConfig {
                    reg_def: rev_reg_def,
                    reg_def_private: rev_reg_def_priv,
                    registry_idx: rev_reg_info.curr_id,
                    status_list: rev_status_list,
                };

                Some(revocation_config)
            }
            _ => None,
        };

        let cred = anoncreds::issuer::create_credential(
            &cred_def,
            &cred_def_private,
            &cred_offer,
            &cred_request,
            cred_values,
            revocation_config,
        )?;

        let rev_reg = cred.rev_reg.as_ref();

        let cred_rev_id =
            if let (Some(rev_reg_id), Some(rev_reg), Some((_, _, _, _, rev_reg_info, _))) =
                (rev_reg_id, rev_reg, revocation_config_parts)
            {
                let cred_rev_id = rev_reg_info.curr_id;
                let str_rev_reg_info = serde_json::to_string(&rev_reg_info)?;

                let rev_reg = AnoncredsRevocationRegistry {
                    value: rev_reg.clone(),
                };
                let str_rev_reg = serde_json::to_string(&rev_reg)?;
                wallet
                    .update_record_value(RecordCategory::RevReg, &rev_reg_id, &str_rev_reg)
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

    #[allow(clippy::too_many_arguments)]
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
        let pres_req: AnoncredsPresentationRequest = proof_req_json.convert(())?;

        let requested_attributes = requested_credentials_json.requested_attributes;
        let requested_predicates = requested_credentials_json.requested_predicates;
        let self_attested_attributes = requested_credentials_json.self_attested_attributes;

        let schemas: HashMap<AnoncredsSchemaId, AnoncredsSchema> = schemas_json.convert(())?;
        let cred_defs: HashMap<AnoncredsCredentialDefinitionId, AnoncredsCredentialDefinition> =
            credential_defs_json.convert(())?;

        let mut present_credentials: PresentCredentials<AnoncredsCredential> =
            PresentCredentials::default();

        let mut proof_details_by_cred_id: HashMap<
            String,
            (
                AnoncredsCredential,
                Option<u64>,
                Option<AnoncredsCredentialRevocationState>,
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
                let credential = self
                    .get_wallet_record_value(wallet, RecordCategory::Cred, cred_id)
                    .await?;

                let (timestamp, rev_state) = get_rev_state(
                    cred_id,
                    &credential,
                    detail.timestamp,
                    revoc_states_json.as_ref(),
                )?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential.convert(())?,
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
                let credential = self
                    .get_wallet_record_value(wallet, RecordCategory::Cred, cred_id)
                    .await?;

                let (timestamp, rev_state) = get_rev_state(
                    cred_id,
                    &credential,
                    detail.timestamp,
                    revoc_states_json.as_ref(),
                )?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential.convert(())?,
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

        let link_secret = self.get_link_secret(wallet, link_secret_id).await?;

        let presentation = anoncreds::prover::create_presentation(
            &pres_req,
            present_credentials,
            Some(self_attested_attributes),
            &link_secret,
            &schemas,
            &cred_defs,
        )?;

        Ok(presentation.convert(())?)
    }

    async fn prover_get_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &CredentialId,
    ) -> VcxAnoncredsResult<RetrievedCredentialInfo> {
        let cred = self
            .get_wallet_record_value(wallet, RecordCategory::Cred, cred_id)
            .await?;
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
        proof_request_json: PresentationRequest,
    ) -> VcxAnoncredsResult<RetrievedCredentials> {
        let requested_attributes = &proof_request_json.value().requested_attributes;
        let requested_predicates = &proof_request_json.value().requested_predicates;

        let mut referents: HashSet<String> = HashSet::new();
        requested_attributes.iter().for_each(|(k, _)| {
            referents.insert(k.to_string());
        });

        requested_predicates.iter().for_each(|(k, _)| {
            referents.insert(k.to_string());
        });

        let mut cred_by_attr = RetrievedCredentials::default();

        for reft in referents {
            let (names, non_revoked, restrictions) = match requested_attributes.get(&reft) {
                Some(attribute_info) => {
                    let attr_names = match (
                        attribute_info.name.to_owned(),
                        attribute_info.names.to_owned(),
                    ) {
                        (Some(name), None) => vec![_normalize_attr_name(&name)],
                        (None, Some(names)) => names
                            .iter()
                            .map(String::as_str)
                            .map(_normalize_attr_name)
                            .collect::<Vec<_>>(),
                        _ => Err(VcxAnoncredsError::InvalidInput(
                            "exactly one of 'name' or 'names' must be present".into(),
                        ))?,
                    };
                    (
                        attr_names,
                        attribute_info.non_revoked.to_owned(),
                        attribute_info.restrictions.to_owned(),
                    )
                }
                None => match requested_predicates.get(&reft) {
                    Some(requested_val) => (
                        vec![requested_val.name.to_owned()],
                        requested_val.non_revoked.to_owned(),
                        requested_val.restrictions.to_owned(),
                    ),
                    None => Err(VcxAnoncredsError::InvalidProofRequest(
                        "Requested attribute or predicate not found in proof request".into(),
                    ))?,
                },
            };

            let credx_creds = self
                ._get_credentials_for_proof_req_for_attr_name(
                    wallet,
                    restrictions.map(serde_json::to_value).transpose()?,
                    names,
                )
                .await?;

            let mut credentials_json = vec![];

            for (cred_id, credx_cred) in credx_creds {
                credentials_json.push(RetrievedCredentialForReferent {
                    cred_info: _make_cred_info(&cred_id, &credx_cred)?,
                    interval: non_revoked.clone(),
                });
            }

            cred_by_attr
                .credentials_by_referent
                .insert(reft, credentials_json);
        }

        Ok(cred_by_attr)
    }

    async fn prover_create_credential_req(
        &self,
        wallet: &impl BaseWallet,
        prover_did: &Did,
        cred_offer_json: CredentialOffer,
        cred_def_json: CredentialDefinition,
        link_secret_id: &LinkSecretId,
    ) -> VcxAnoncredsResult<(CredentialRequest, CredentialRequestMetadata)> {
        let cred_def: AnoncredsCredentialDefinition = cred_def_json.convert(())?;
        let credential_offer: AnoncredsCredentialOffer = cred_offer_json.convert(())?;
        let link_secret = self.get_link_secret(wallet, link_secret_id).await?;

        let (cred_req, cred_req_metadata) = anoncreds::prover::create_credential_request(
            None,
            Some(prover_did.did()),
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
        rev_status_list: RevocationStatusList,
        cred_rev_id: u32,
    ) -> VcxAnoncredsResult<CredentialRevocationState> {
        let revoc_reg_def: AnoncredsRevocationRegistryDefinition = rev_reg_def_json.convert(())?;
        let tails_file_hash = revoc_reg_def.value.tails_hash.as_str();

        let mut tails_file_path = std::path::PathBuf::new();
        tails_file_path.push(tails_dir);
        tails_file_path.push(tails_file_hash);

        let tails_path = tails_file_path.to_str().ok_or_else(|| {
            VcxAnoncredsError::InvalidOption("tails file is not an unicode string".into())
        })?;

        let rev_state = anoncreds::prover::create_or_update_revocation_state(
            tails_path,
            &revoc_reg_def,
            &rev_status_list.convert(())?,
            cred_rev_id,
            None,
            None,
        )?;

        Ok(rev_state.convert(())?)
    }

    async fn prover_store_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_req_metadata: CredentialRequestMetadata,
        unprocessed_cred: Credential,
        schema: Schema,
        cred_def: CredentialDefinition,
        rev_reg_def: Option<RevocationRegistryDefinition>,
    ) -> VcxAnoncredsResult<CredentialId> {
        let mut credential: AnoncredsCredential = unprocessed_cred.convert(())?;

        let cred_request_metadata: AnoncredsCredentialRequestMetadata =
            cred_req_metadata.convert(())?;
        let link_secret_id = &cred_request_metadata.link_secret_name;
        let link_secret = self.get_link_secret(wallet, link_secret_id).await?;
        let cred_def: AnoncredsCredentialDefinition = cred_def.convert(())?;
        let rev_reg_def: Option<AnoncredsRevocationRegistryDefinition> =
            if let Some(rev_reg_def_json) = rev_reg_def {
                Some(rev_reg_def_json.convert(())?)
            } else {
                None
            };

        anoncreds::prover::process_credential(
            &mut credential,
            &cred_request_metadata,
            &link_secret,
            &cred_def,
            rev_reg_def.as_ref(),
        )?;

        let schema_id = &credential.schema_id;
        let cred_def_id = &credential.cred_def_id;
        let issuer_did = &cred_def.issuer_id;

        let schema_issuer_did = schema.issuer_id;
        let schema_name = schema.name;
        let schema_version = schema.version;

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
            let attr_name = _normalize_attr_name(raw_attr_name.as_str());
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

    async fn prover_delete_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &CredentialId,
    ) -> VcxAnoncredsResult<()> {
        Ok(wallet.delete_record(RecordCategory::Cred, cred_id).await?)
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

        let secret = anoncreds::prover::create_link_secret()?;
        let ms_decimal = TryInto::<String>::try_into(secret).map_err(|err| {
            VcxAnoncredsError::UrsaError(format!(
                "Failed convert BigNumber to decimal string: {}",
                err
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

    async fn issuer_create_schema(
        &self,
        issuer_did: &Did,
        name: &str,
        version: &str,
        attrs: AttributeNames,
    ) -> VcxAnoncredsResult<Schema> {
        let schema = anoncreds::issuer::create_schema(
            name,
            version,
            IssuerId::new(issuer_did.to_string()).unwrap(),
            attrs.convert(())?,
        )?;
        let schema_id = make_schema_id(issuer_did, name, version);
        Ok(schema.convert((schema_id.to_string(),))?)
    }

    async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
        cred_rev_id: u32,
        ledger_rev_reg_delta_json: RevocationRegistryDelta,
    ) -> VcxAnoncredsResult<()> {
        let rev_reg_def: RevocationRegistryDefinition = self
            .get_wallet_record_value(wallet, RecordCategory::RevRegDef, &rev_reg_id.to_string())
            .await?;

        let last_rev_reg_delta_stored = self.get_rev_reg_delta(wallet, rev_reg_id).await?;
        let last_rev_reg_delta = last_rev_reg_delta_stored
            .clone()
            .unwrap_or(ledger_rev_reg_delta_json.clone());

        let prev_accum = ledger_rev_reg_delta_json.value.accum;

        let current_time = OffsetDateTime::now_utc().unix_timestamp() as u64;
        let rev_status_list = from_revocation_registry_delta_to_revocation_status_list(
            &last_rev_reg_delta.value,
            current_time,
            &rev_reg_def.id,
            rev_reg_def.value.max_cred_num as usize,
            rev_reg_def.issuer_id.clone(),
        )?;

        let cred_def = self
            .get_wallet_record_value(
                wallet,
                RecordCategory::CredDef,
                &rev_reg_def.cred_def_id.to_string(),
            )
            .await?;

        let rev_reg_def_priv = self
            .get_wallet_record_value(
                wallet,
                RecordCategory::RevRegDefPriv,
                &rev_reg_id.to_string(),
            )
            .await?;

        let updated_rev_status_list = anoncreds::issuer::update_revocation_status_list(
            &cred_def,
            &rev_reg_def.convert(())?,
            &rev_reg_def_priv,
            &rev_status_list.convert(())?,
            None,
            Some(vec![cred_rev_id].into_iter().collect()),
            None,
        )?;

        let updated_revocation_registry_delta =
            from_revocation_status_list_to_revocation_registry_delta(
                &updated_rev_status_list.convert(())?,
                Some(prev_accum),
            )?;
        let updated_revocation_registry_delta_str =
            serde_json::to_string(&updated_revocation_registry_delta)?;

        if last_rev_reg_delta_stored.is_some() {
            wallet
                .update_record_value(
                    RecordCategory::RevRegDelta,
                    &rev_reg_id.to_string(),
                    &updated_revocation_registry_delta_str,
                )
                .await?;
        } else {
            let record = Record::builder()
                .name(rev_reg_id.to_string())
                .category(RecordCategory::RevRegDelta)
                .value(updated_revocation_registry_delta_str)
                .build();
            wallet.add_record(record).await?;
        };

        Ok(())
    }

    async fn get_rev_reg_delta(
        &self,
        wallet: &impl BaseWallet,
        rev_reg_id: &RevocationRegistryDefinitionId,
    ) -> VcxAnoncredsResult<Option<RevocationRegistryDelta>> {
        let res_rev_reg_delta = self
            .get_wallet_record_value::<RevocationRegistryDelta>(
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
        Ok(anoncreds::verifier::generate_nonce()?.convert(())?)
    }
}

fn get_rev_state(
    cred_id: &str,
    credential: &Credential,
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
    cred: &Credential,
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
        schema_id: cred.schema_id.clone(),
        cred_def_id: cred.cred_def_id.clone(),
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

pub fn make_schema_id(did: &Did, name: &str, version: &str) -> SchemaId {
    let prefix = did
        .method()
        .map(|method| format!("schema:{}:", method))
        .unwrap_or_default();
    let id = format!("{}{}:2:{}:{}", prefix, did, name, version);
    SchemaId::new(id).unwrap()
}

pub fn make_credential_definition_id(
    origin_did: &Did,
    schema_id: &SchemaId,
    schema_seq_no: Option<u32>,
    tag: &str,
    signature_type: SignatureType,
) -> CredentialDefinitionId {
    let signature_type = match signature_type {
        SignatureType::CL => CL_SIGNATURE_TYPE,
    };

    let tag = if tag.is_empty() {
        String::new()
    } else {
        format!(":{}", tag)
    };

    let prefix = origin_did
        .method()
        .map(|method| format!("creddef:{}:", method))
        .unwrap_or_default();

    let schema_infix_id = schema_seq_no
        .map(|n| n.to_string())
        .unwrap_or(schema_id.to_string());

    let id = format!(
        "{}{}:3:{}:{}{}",
        prefix,
        origin_did,
        signature_type,
        schema_infix_id,
        tag // prefix, origin_did, signature_type, schema_id, tag
    );

    CredentialDefinitionId::new(id).unwrap()
}

fn make_revocation_registry_id(
    origin_did: &Did,
    cred_def_id: &CredentialDefinitionId,
    tag: &str,
    rev_reg_type: RegistryType,
) -> VcxAnoncredsResult<RevocationRegistryDefinitionId> {
    // Must use unchecked as anoncreds doesn't expose validation error
    Ok(RevocationRegistryDefinitionId::new(format!(
        "{}{}:4:{}:{}:{}",
        origin_did
            .method()
            .map_or(Default::default(), |method| format!("revreg:{}:", method)),
        origin_did,
        cred_def_id.0,
        match rev_reg_type {
            RegistryType::CL_ACCUM => CL_ACCUM,
        },
        tag
    ))
    .unwrap())
}

pub fn schema_parts(id: &str) -> Option<(Option<&str>, Did, String, String)> {
    let parts = id.split_terminator(':').collect::<Vec<&str>>();

    if parts.len() == 1 {
        // 1
        return None;
    }

    if parts.len() == 4 {
        // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
        let did = parts[0].to_string();
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let name = parts[2].to_string();
        let version = parts[3].to_string();
        return Some((None, did, name, version));
    }

    if parts.len() == 8 {
        // schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
        let method = parts[1];
        let did = parts[2..5].join(":");
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let name = parts[6].to_string();
        let version = parts[7].to_string();
        return Some((Some(method), did, name, version));
    }

    None
}
