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
        CredentialRevocationConfig, CredentialRevocationState,
        CredentialValues as AnoncredsCredentialValues, LinkSecret, PresentCredentials,
        Presentation as AnoncredsPresentation, PresentationRequest as AnoncredsPresentationRequest,
        RegistryType, RevocationRegistry as AnoncredsRevocationRegistry,
        RevocationRegistryDefinition as AnoncredsRevocationRegistryDefinition,
        RevocationStatusList, SignatureType,
    },
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
        rev_reg_delta::{RevocationRegistryDelta, RevocationRegistryDeltaValue},
        schema::Schema,
    },
    messages::{
        cred_offer::CredentialOffer,
        cred_request::{CredentialRequest, CredentialRequestMetadata},
        cred_selection::{RetrievedCredentials, SelectedCredentialInfo},
        credential::{Credential, CredentialValues},
        nonce::Nonce,
        pres_request::PresentationRequest,
        presentation::Presentation,
    },
};
use async_trait::async_trait;
use bitvec::bitvec;
use did_parser::Did;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use time::OffsetDateTime;
use uuid::Uuid;

use super::base_anoncreds::BaseAnonCreds;
use crate::{
    anoncreds::anoncreds::type_conversion::Convert,
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    utils::{
        constants::ATTRS,
        json::{AsTypeOrDeserializationError, TryGetIndex},
    },
    wallet::base_wallet::{record::Record, search_filter::SearchFilter, BaseWallet},
};

pub const CATEGORY_LINK_SECRET: &str = "VCX_LINK_SECRET";

pub const CATEGORY_CREDENTIAL: &str = "VCX_CREDENTIAL";
pub const CATEGORY_CRED_DEF: &str = "VCX_CRED_DEF";
pub const CATEGORY_CRED_KEY_CORRECTNESS_PROOF: &str = "VCX_CRED_KEY_CORRECTNESS_PROOF";
pub const CATEGORY_CRED_DEF_PRIV: &str = "VCX_CRED_DEF_PRIV";
pub const CATEGORY_CRED_SCHEMA: &str = "VCX_CRED_SCHEMA";

pub const CATEGORY_CRED_MAP_SCHEMA_ID: &str = "VCX_CRED_MAP_SCHEMA_ID";

pub const CATEGORY_REV_REG: &str = "VCX_REV_REG";
pub const CATEGORY_REV_REG_DELTA: &str = "VCX_REV_REG_DELTA";
pub const CATEGORY_REV_REG_INFO: &str = "VCX_REV_REG_INFO";
pub const CATEGORY_REV_REG_DEF: &str = "VCX_REV_REG_DEF";
pub const CATEGORY_REV_REG_DEF_PRIV: &str = "VCX_REV_REG_DEF_PRIV";

fn from_revocation_registry_delta_to_revocation_status_list(
    delta: &RevocationRegistryDeltaValue,
    rev_reg_def: &AnoncredsRevocationRegistryDefinition,
    rev_reg_def_id: &RevocationRegistryDefinitionId,
    timestamp: Option<u64>,
    issuance_by_default: bool,
) -> VcxCoreResult<RevocationStatusList> {
    let default_state = if issuance_by_default { 0 } else { 1 };
    let mut revocation_list = bitvec![default_state; rev_reg_def.value.max_cred_num as usize];

    for issued in &delta.issued {
        revocation_list.insert(*issued as usize, false);
    }

    for revoked in &delta.revoked {
        revocation_list.insert(*revoked as usize, true);
    }

    let accum = delta.accum.into();

    RevocationStatusList::new(
        Some(&rev_reg_def_id.to_string()),
        rev_reg_def.issuer_id.clone(),
        revocation_list,
        Some(accum),
        timestamp,
    )
    .map_err(Into::into)
}

fn from_revocation_status_list_to_revocation_registry_delta(
    rev_status_list: &RevocationStatusList,
    prev_accum: Option<Accumulator>,
) -> VcxCoreResult<RevocationRegistryDelta> {
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
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidState,
                "Revocation registry delta cannot be created from revocation status list without \
                 accumulator",
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

#[derive(Debug)]
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
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<LinkSecret> {
        let ms_decimal = wallet
            .get_record(CATEGORY_LINK_SECRET, link_secret_id)
            .await?;

        Ok(ms_decimal.value().try_into().unwrap())
    }

    async fn _get_credential(
        &self,
        wallet: &impl BaseWallet,
        credential_id: &str,
    ) -> VcxCoreResult<Credential> {
        let cred_record = wallet
            .get_record(CATEGORY_CREDENTIAL, credential_id)
            .await?;

        let credential: Credential = serde_json::from_str(cred_record.value())?;

        Ok(credential)
    }

    async fn _get_credentials(
        wallet: &impl BaseWallet,
        wql: &str,
    ) -> VcxCoreResult<Vec<(String, Credential)>> {
        let records = wallet
            .search_record(
                CATEGORY_CREDENTIAL,
                Some(SearchFilter::JsonFilter(wql.into())),
            )
            .await?;

        let id_cred_tuple_list: VcxCoreResult<Vec<(String, Credential)>> = records
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
        restrictions: Option<&Value>,
        attr_names: Vec<String>,
    ) -> VcxCoreResult<Vec<(String, Credential)>> {
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
impl BaseAnonCreds for Anoncreds {
    async fn verifier_verify_proof(
        &self,
        proof_request_json: PresentationRequest,
        proof_json: Presentation,
        schemas_json: HashMap<SchemaId, Schema>,
        credential_defs_json: HashMap<CredentialDefinitionId, CredentialDefinition>,
        rev_reg_defs_json: Option<
            HashMap<RevocationRegistryDefinitionId, RevocationRegistryDefinition>,
        >,
        rev_regs_json: Option<
            HashMap<RevocationRegistryDefinitionId, HashMap<u64, RevocationRegistry>>,
        >,
    ) -> VcxCoreResult<bool> {
        let presentation: AnoncredsPresentation = proof_json.convert(())?;
        let pres_req: AnoncredsPresentationRequest = proof_request_json.convert(())?;

        let schemas: HashMap<AnoncredsSchemaId, AnoncredsSchema> = schemas_json.convert(())?;

        let cred_defs: HashMap<AnoncredsCredentialDefinitionId, AnoncredsCredentialDefinition> =
            credential_defs_json.convert(())?;

        let rev_reg_defs: Option<
            HashMap<AnoncredsRevocationRegistryDefinitionId, AnoncredsRevocationRegistryDefinition>,
        > = rev_reg_defs_json.map(|v| v.convert(())).transpose()?;

        Ok(anoncreds::verifier::verify_presentation(
            &presentation,
            &pres_req,
            &schemas,
            &cred_defs,
            rev_reg_defs.as_ref(),
            rev_regs_json.map(|r| r.convert(())).transpose()?,
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
    ) -> VcxCoreResult<(
        RevocationRegistryDefinitionId,
        RevocationRegistryDefinition,
        RevocationRegistry,
    )> {
        let mut tails_writer = TailsFileWriter::new(Some(tails_dir.to_str().unwrap().to_string()));

        let cred_def: AnoncredsCredentialDefinition = self
            .get_wallet_record_value(wallet, CATEGORY_CRED_DEF, &cred_def_id.to_string())
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

        let opt_rev_reg: Option<CryptoRevocationRegistry> = (&rev_status_list).try_into().unwrap();
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
            .category(CATEGORY_REV_REG_INFO.to_string())
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

        let rev_reg = AnoncredsRevocationRegistry { value: rev_reg };
        let str_rev_reg = serde_json::to_string(&rev_reg)?;
        let record = Record::builder()
            .name(rev_reg_id.0.clone())
            .category(CATEGORY_REV_REG.to_string())
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
        tag: &str,
        signature_type: Option<&str>,
        config_json: &str,
    ) -> VcxCoreResult<CredentialDefinition> {
        let sig_type = signature_type
            .map(serde_json::from_str)
            .unwrap_or(Ok(SignatureType::CL))?;
        let config = serde_json::from_str(config_json)?;

        let cred_def_id =
            make_credential_definition_id(issuer_did, schema_id, schema_json.seq_no, tag, sig_type);

        // If cred def already exists, return it
        if let Ok(cred_def) = self
            .get_wallet_record_value(wallet, CATEGORY_CRED_DEF, &cred_def_id.0)
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
                tag,
                sig_type,
                config,
            )?;

        let mut cred_def_val = serde_json::to_value(&cred_def)?;
        cred_def_val
            .as_object_mut()
            .map(|v| v.insert("id".to_owned(), cred_def_id.to_string().into()));

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
            .name(schema_id.to_string())
            .category(CATEGORY_CRED_SCHEMA.to_string())
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
            .category(CATEGORY_CRED_MAP_SCHEMA_ID.to_string())
            // .value(schema_id.to_string())
            .value(json!({"schemaId": schema_id}).to_string())
            .build();
        wallet.add_record(record).await?;

        Ok(cred_def.convert((cred_def_id.to_string(),))?)
    }

    async fn issuer_create_credential_offer(
        &self,
        wallet: &impl BaseWallet,
        cred_def_id: &CredentialDefinitionId,
    ) -> VcxCoreResult<CredentialOffer> {
        let correctness_proof = self
            .get_wallet_record_value(
                wallet,
                CATEGORY_CRED_KEY_CORRECTNESS_PROOF,
                &cred_def_id.to_string(),
            )
            .await?;

        let schema_id_value = self
            .get_wallet_record_value::<Value>(
                wallet,
                CATEGORY_CRED_MAP_SCHEMA_ID,
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
    ) -> VcxCoreResult<(Credential, Option<u32>)> {
        let cred_offer: AnoncredsCredentialOffer = cred_offer_json.convert(())?;
        let cred_request: AnoncredsCredentialRequest = cred_req_json.convert(())?;
        let cred_values: AnoncredsCredentialValues = cred_values_json.convert(())?;

        let cred_def_id = &cred_offer.cred_def_id.0;

        let cred_def = self
            .get_wallet_record_value(wallet, CATEGORY_CRED_DEF, cred_def_id)
            .await?;

        let cred_def_private = self
            .get_wallet_record_value(wallet, CATEGORY_CRED_DEF_PRIV, cred_def_id)
            .await?;

        let rev_reg_id = rev_reg_id.map(ToString::to_string);
        let mut revocation_config_parts = match (tails_dir, &rev_reg_id) {
            (Some(tails_dir), Some(rev_reg_def_id)) => {
                let rev_reg_def: AnoncredsRevocationRegistryDefinition = self
                    .get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF, rev_reg_def_id)
                    .await?;

                let rev_reg_def_priv = self
                    .get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF_PRIV, rev_reg_def_id)
                    .await?;

                let rev_reg: AnoncredsRevocationRegistry = self
                    .get_wallet_record_value(wallet, CATEGORY_REV_REG, rev_reg_def_id)
                    .await?;
                let rev_reg_info: RevocationRegistryInfo = self
                    .get_wallet_record_value(wallet, CATEGORY_REV_REG_INFO, rev_reg_def_id)
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
                    return Err(AriesVcxCoreError::from_msg(
                        AriesVcxCoreErrorKind::ActionNotSupported,
                        "The revocation registry is full",
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
                    .update_record_value(CATEGORY_REV_REG, &rev_reg_id, &str_rev_reg)
                    .await?;

                wallet
                    .update_record_value(CATEGORY_REV_REG_INFO, &rev_reg_id, &str_rev_reg_info)
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
        requested_credentials_json: &str,
        master_secret_id: &str,
        schemas_json: HashMap<SchemaId, Schema>,
        credential_defs_json: HashMap<CredentialDefinitionId, CredentialDefinition>,
        revoc_states_json: Option<&str>,
    ) -> VcxCoreResult<Presentation> {
        let pres_req: AnoncredsPresentationRequest = proof_req_json.convert(())?;

        let requested_credentials: Value = serde_json::from_str(requested_credentials_json)?;
        let requested_attributes = (&requested_credentials).try_get("requested_attributes")?;

        let requested_predicates = (&requested_credentials).try_get("requested_predicates")?;
        let self_attested_attributes = requested_credentials.get("self_attested_attributes");

        let rev_states: Option<Value> = if let Some(revoc_states_json) = revoc_states_json {
            Some(serde_json::from_str(revoc_states_json)?)
        } else {
            None
        };

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
                let credential = self._get_credential(wallet, cred_id).await?;

                let (timestamp, rev_state) =
                    get_rev_state(cred_id, &credential, detail, rev_states.as_ref())?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential.convert(())?,
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
                let credential = self._get_credential(wallet, cred_id).await?;

                let (timestamp, rev_state) =
                    get_rev_state(cred_id, &credential, detail, rev_states.as_ref())?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (
                        credential.convert(())?,
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

        let link_secret = self.get_link_secret(wallet, master_secret_id).await?;

        let presentation = anoncreds::prover::create_presentation(
            &pres_req,
            present_credentials,
            self_attested,
            &link_secret,
            &schemas,
            &cred_defs,
        )?;

        Ok(presentation.convert(())?)
    }

    async fn prover_get_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<SelectedCredentialInfo> {
        let cred = self._get_credential(wallet, cred_id).await?;
        let cred_info = _make_cred_info(cred_id, &cred)?;
        Ok(serde_json::from_value(cred_info)?)
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
        proof_request_json: PresentationRequest,
    ) -> VcxCoreResult<RetrievedCredentials> {
        let proof_req_v: Value = serde_json::to_value(proof_request_json).map_err(|e| {
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

        Ok(serde_json::from_value(cred_by_attr)?)
    }

    async fn prover_create_credential_req(
        &self,
        wallet: &impl BaseWallet,
        prover_did: &Did,
        cred_offer_json: CredentialOffer,
        cred_def_json: CredentialDefinition,
        master_secret_id: &str,
    ) -> VcxCoreResult<(CredentialRequest, CredentialRequestMetadata)> {
        let cred_def: AnoncredsCredentialDefinition = cred_def_json.convert(())?;
        let credential_offer: AnoncredsCredentialOffer = cred_offer_json.convert(())?;
        let link_secret = self.get_link_secret(wallet, master_secret_id).await?;

        let (cred_req, cred_req_metadata) = anoncreds::prover::create_credential_request(
            None,
            Some(prover_did.did()),
            &cred_def,
            &link_secret,
            master_secret_id,
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
    ) -> VcxCoreResult<String> {
        let cred_def_id = rev_reg_def_json.cred_def_id.to_string();
        let max_cred_num = rev_reg_def_json.value.max_cred_num;
        let rev_reg_def_id = rev_reg_def_json.id.to_string();
        let (_cred_def_method, issuer_did, _signature_type, _schema_num, _tag) =
            cred_def_parts(&cred_def_id).ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidSchema,
                format!("Could not process cred_def_id {cred_def_id} as parts."),
            ))?;

        let revoc_reg_def: AnoncredsRevocationRegistryDefinition = rev_reg_def_json.convert(())?;
        let tails_file_hash = revoc_reg_def.value.tails_hash.as_str();

        let mut tails_file_path = std::path::PathBuf::new();
        tails_file_path.push(tails_dir);
        tails_file_path.push(tails_file_hash);

        let tails_path = tails_file_path.to_str().ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidOption,
                "tails file is not an unicode string",
            )
        })?;

        let RevocationRegistryDeltaValue { accum, revoked, .. } = rev_reg_delta_json.value;

        let issuer_id = IssuerId::new(issuer_did.did()).unwrap();
        let mut revocation_list = bitvec!(0; max_cred_num as usize);
        revoked.into_iter().for_each(|id| {
            revocation_list
                .get_mut(id as usize)
                .map(|mut b| *b = true)
                .unwrap_or_default()
        });
        let registry = CryptoRevocationRegistry { accum };

        // let rev_status_list = create_revocation_status_list(
        //     &cred_def,
        //     RevocationRegistryDefinitionId::new_unchecked(issuer_id),
        //     &revoc_reg_def,
        //     rev_reg_priv, // No way to construct this from revocation registry currently
        //     true,
        //     Some(timestamp),
        // );

        // TODO: Made public, should find a better way
        let rev_status_list = RevocationStatusList::new(
            Some(&rev_reg_def_id),
            issuer_id,
            revocation_list,
            Some(registry),
            Some(timestamp),
        )?;

        let rev_state = anoncreds::prover::create_or_update_revocation_state(
            tails_path,
            &revoc_reg_def,
            &rev_status_list,
            cred_rev_id,
            None,
            None,
        )?;

        Ok(serde_json::to_string(&rev_state)?)
    }

    async fn prover_store_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_req_metadata_json: CredentialRequestMetadata,
        cred_json: Credential,
        cred_def_json: CredentialDefinition,
        rev_reg_def_json: Option<RevocationRegistryDefinition>,
    ) -> VcxCoreResult<String> {
        let mut credential: AnoncredsCredential = cred_json.convert(())?;

        let cred_def_id = credential.cred_def_id.to_string();
        let (_cred_def_method, issuer_did, _signature_type, _schema_num, _tag) =
            cred_def_parts(&cred_def_id).ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidSchema,
                "Could not process credential.cred_def_id as parts.",
            ))?;

        let cred_request_metadata: AnoncredsCredentialRequestMetadata =
            cred_req_metadata_json.convert(())?;
        let link_secret_id = &cred_request_metadata.link_secret_name;
        let link_secret = self.get_link_secret(wallet, link_secret_id).await?;
        let cred_def: AnoncredsCredentialDefinition = cred_def_json.convert(())?;
        let rev_reg_def: Option<AnoncredsRevocationRegistryDefinition> =
            if let Some(rev_reg_def_json) = rev_reg_def_json {
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

        let (_schema_method, schema_issuer_did, schema_name, schema_version) =
            schema_parts(schema_id.0.as_str()).ok_or(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidSchema,
                format!("Could not process credential.schema_id {schema_id} as parts."),
            ))?;

        let mut tags = json!({
            "schema_id": schema_id.0,
            "schema_issuer_did": schema_issuer_did.did(),
            "schema_name": schema_name,
            "schema_version": schema_version,
            "issuer_did": issuer_did.did(),
            "cred_def_id": cred_def_id
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

        let credential_id = Uuid::new_v4().to_string();

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

    async fn prover_delete_credential(
        &self,
        wallet: &impl BaseWallet,
        cred_id: &str,
    ) -> VcxCoreResult<()> {
        wallet.delete_record(CATEGORY_CREDENTIAL, cred_id).await
    }

    async fn prover_create_link_secret(
        &self,
        wallet: &impl BaseWallet,
        link_secret_id: &str,
    ) -> VcxCoreResult<()> {
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

        let secret = anoncreds::prover::create_link_secret()?;
        let ms_decimal = TryInto::<String>::try_into(secret).map_err(|err| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::UrsaError,
                format!("Failed convert BigNumber to decimal string: {}", err),
            )
        })?;

        let record = Record::builder()
            .name(link_secret_id.into())
            .category(CATEGORY_LINK_SECRET.into())
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
        attrs: &str,
    ) -> VcxCoreResult<Schema> {
        let attr_names = serde_json::from_str(attrs)?;

        let schema = anoncreds::issuer::create_schema(
            name,
            version,
            IssuerId::new(issuer_did.to_string()).unwrap(),
            attr_names,
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
    ) -> VcxCoreResult<()> {
        let rev_reg_def: RevocationRegistryDefinition = self
            .get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF, &rev_reg_id.to_string())
            .await?;

        let last_rev_reg_delta_stored = self.get_rev_reg_delta(wallet, rev_reg_id).await?;
        let last_rev_reg_delta = last_rev_reg_delta_stored
            .clone()
            .unwrap_or(ledger_rev_reg_delta_json.clone());

        let prev_accum = ledger_rev_reg_delta_json.value.accum;

        let current_time = OffsetDateTime::now_utc().unix_timestamp() as u64;
        let rev_status_list = from_revocation_registry_delta_to_revocation_status_list(
            &last_rev_reg_delta.value,
            &rev_reg_def.clone().convert(())?,
            rev_reg_id,
            Some(current_time),
            true,
        )?;

        let cred_def = self
            .get_wallet_record_value(
                wallet,
                CATEGORY_CRED_DEF,
                &rev_reg_def.cred_def_id.to_string(),
            )
            .await?;

        let rev_reg_def_priv = self
            .get_wallet_record_value(wallet, CATEGORY_REV_REG_DEF_PRIV, &rev_reg_id.to_string())
            .await?;

        let updated_rev_status_list = anoncreds::issuer::update_revocation_status_list(
            &cred_def,
            &rev_reg_def.convert(())?,
            &rev_reg_def_priv,
            &rev_status_list,
            None,
            Some(vec![cred_rev_id].into_iter().collect()),
            None,
        )?;

        let updated_revocation_registry_delta =
            from_revocation_status_list_to_revocation_registry_delta(
                &updated_rev_status_list,
                Some(prev_accum),
            )?;
        let updated_revocation_registry_delta_str =
            serde_json::to_string(&updated_revocation_registry_delta)?;

        if last_rev_reg_delta_stored.is_some() {
            wallet
                .update_record_value(
                    CATEGORY_REV_REG_DELTA,
                    &rev_reg_id.to_string(),
                    &updated_revocation_registry_delta_str,
                )
                .await?;
        } else {
            let record = Record::builder()
                .name(rev_reg_id.to_string())
                .category(CATEGORY_REV_REG_DELTA.into())
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
    ) -> VcxCoreResult<Option<RevocationRegistryDelta>> {
        let res_rev_reg_delta = self
            .get_wallet_record_value::<RevocationRegistryDelta>(
                wallet,
                CATEGORY_REV_REG_DELTA,
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
    ) -> VcxCoreResult<()> {
        if self.get_rev_reg_delta(wallet, rev_reg_id).await?.is_some() {
            wallet
                .delete_record(CATEGORY_REV_REG_DELTA, &rev_reg_id.to_string())
                .await?;
        }

        Ok(())
    }

    async fn generate_nonce(&self) -> VcxCoreResult<Nonce> {
        Ok(anoncreds::verifier::generate_nonce()?.convert(())?)
    }
}

fn get_rev_state(
    cred_id: &str,
    credential: &Credential,
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

fn _make_cred_info(credential_id: &str, cred: &Credential) -> VcxCoreResult<Value> {
    let cred_sig = serde_json::to_value(&cred.signature)?;

    let rev_info = cred_sig.get("r_credential");

    let schema_id = &cred.schema_id.0;
    let cred_def_id = &cred.cred_def_id.0;
    let rev_reg_id = cred.rev_reg_id.as_ref().map(|x| x.0.to_string());
    let cred_rev_id: Option<u32> = rev_info
        .and_then(|x| x.get("i"))
        .and_then(|i| i.as_u64().map(|i| i as u32));

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
) -> VcxCoreResult<RevocationRegistryDefinitionId> {
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

pub fn cred_def_parts(id: &str) -> Option<(Option<&str>, Did, String, SchemaId, String)> {
    let parts = id.split_terminator(':').collect::<Vec<&str>>();

    if parts.len() == 4 {
        // Th7MpTaRZVRYnPiabds81Y:3:CL:1
        let did = parts[0].to_string();
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let signature_type = parts[2].to_string();
        let schema_id = parts[3].to_string();
        let tag = String::new();
        return Some((None, did, signature_type, SchemaId(schema_id), tag));
    }

    if parts.len() == 5 {
        // Th7MpTaRZVRYnPiabds81Y:3:CL:1:tag
        let did = parts[0].to_string();
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let signature_type = parts[2].to_string();
        let schema_id = parts[3].to_string();
        let tag = parts[4].to_string();
        return Some((None, did, signature_type, SchemaId(schema_id), tag));
    }

    if parts.len() == 7 {
        // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
        let did = parts[0].to_string();
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let signature_type = parts[2].to_string();
        let schema_id = parts[3..7].join(":");
        let tag = String::new();
        return Some((None, did, signature_type, SchemaId(schema_id), tag));
    }

    if parts.len() == 8 {
        // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
        let did = parts[0].to_string();
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let signature_type = parts[2].to_string();
        let schema_id = parts[3..7].join(":");
        let tag = parts[7].to_string();
        return Some((None, did, signature_type, SchemaId(schema_id), tag));
    }

    if parts.len() == 9 {
        // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:3:tag
        let method = parts[1];
        let did = parts[2..5].join(":");
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let signature_type = parts[6].to_string();
        let schema_id = parts[7].to_string();
        let tag = parts[8].to_string();
        return Some((Some(method), did, signature_type, SchemaId(schema_id), tag));
    }

    if parts.len() == 16 {
        // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:
        // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
        let method = parts[1];
        let did = parts[2..5].join(":");
        let Ok(did) = Did::parse(did) else {
            return None;
        };
        let signature_type = parts[6].to_string();
        let schema_id = parts[7..15].join(":");
        let tag = parts[15].to_string();
        return Some((Some(method), did, signature_type, SchemaId(schema_id), tag));
    }

    None
}
