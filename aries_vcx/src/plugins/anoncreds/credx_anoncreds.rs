use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_trait::async_trait;
use credx::{
    types::{
        Credential as CredxCredential, CredentialDefinition, CredentialDefinitionId, CredentialOffer,
        CredentialRequestMetadata, CredentialRevocationState, DidValue, MasterSecret, PresentCredentials, Presentation,
        PresentationRequest, RevocationRegistry, RevocationRegistryDefinition, RevocationRegistryDelta,
        RevocationRegistryId, Schema, SchemaId,
    },
    ursa::{bn::BigNumber, cl::MasterSecret as UrsaMasterSecret},
};
use indy_credx as credx;
use serde_json::Value;

use super::base_anoncreds::BaseAnonCreds;
use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    plugins::wallet::base_wallet::AsyncFnIteratorCollect,
    utils::{
        constants::ATTRS,
        json::{AsTypeOrDeserializationError, TryGetIndex},
        uuid::uuid,
    },
};

const CATEGORY_CREDENTIAL: &str = "VCX_CREDENTIAL";
const CATEGORY_LINK_SECRET: &str = "VCX_LINK_SECRET";

#[derive(Debug)]
pub struct IndyCredxAnonCreds {
    profile: Arc<dyn Profile>,
}

impl IndyCredxAnonCreds {
    pub fn new(profile: Arc<dyn Profile>) -> Self {
        IndyCredxAnonCreds { profile }
    }

    async fn get_link_secret(&self, link_secret_id: &str) -> VcxResult<MasterSecret> {
        let wallet = self.profile.inject_wallet();

        let record = wallet
            .get_wallet_record(CATEGORY_LINK_SECRET, link_secret_id, "{}")
            .await?;

        let record: Value = serde_json::from_str(&record)?;

        let ms_value = (&record).try_get("value")?;
        let ms_decimal = ms_value.try_as_str()?;
        let ms_bn: BigNumber = BigNumber::from_dec(ms_decimal).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::UrsaError,
                format!("Failed to create BigNumber, UrsaErrorKind: {}", err.kind()),
            )
        })?;
        let ursa_ms: UrsaMasterSecret = serde_json::from_value(json!({ "ms": ms_bn }))?;

        Ok(MasterSecret { value: ursa_ms })
    }

    async fn _get_credential(&self, credential_id: &str) -> VcxResult<CredxCredential> {
        let wallet = self.profile.inject_wallet();

        let cred_record = wallet
            .get_wallet_record(CATEGORY_CREDENTIAL, credential_id, "{}")
            .await?;
        let cred_record: Value = serde_json::from_str(&cred_record)?;
        let cred_record_value = (&cred_record).try_get("value")?;

        let cred_json = cred_record_value.try_as_str()?;

        let credential: CredxCredential = serde_json::from_str(cred_json)?;

        Ok(credential)
    }

    async fn _get_credentials(&self, wql: &str) -> VcxResult<Vec<(String, CredxCredential)>> {
        let wallet = self.profile.inject_wallet();

        let mut record_iterator = wallet.iterate_wallet_records(CATEGORY_CREDENTIAL, wql, "{}").await?;
        let records = record_iterator.collect().await?;

        let id_cred_tuple_list: VcxResult<Vec<(String, CredxCredential)>> = records
            .iter()
            .map(|record| {
                let cred_record: Value = serde_json::from_str(record)?;

                let cred_record_id = (&cred_record).try_get("id")?.try_as_str()?.to_string();
                let cred_record_value = (&cred_record).try_get("value")?;

                let cred_json = cred_record_value.try_as_str()?;

                let credential: CredxCredential = serde_json::from_str(cred_json)?;

                Ok((cred_record_id, credential))
            })
            .collect();

        id_cred_tuple_list
    }

    async fn _get_credentials_for_proof_req_for_attr_name(
        &self,
        restrictions: Option<&Value>,
        attr_name: &str,
    ) -> VcxResult<Vec<(String, CredxCredential)>> {
        let attr_marker_tag_name = _format_attribute_as_marker_tag_name(attr_name);

        let wql_attr_query = json!({
            attr_marker_tag_name: "1"
        });

        let restrictions = restrictions.map(|x| x.to_owned());

        let wql_query = if let Some(restrictions) = restrictions {
            match restrictions {
                Value::Array(mut arr) => {
                    arr.push(wql_attr_query);
                    json!({ "$and": arr })
                }
                Value::Object(obj) => json!({ "$and": vec![wql_attr_query, Value::Object(obj)] }),
                _ => wql_attr_query,
            }
        } else {
            wql_attr_query
        };

        let wql_query = serde_json::to_string(&wql_query)?;

        self._get_credentials(&wql_query).await
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
    ) -> VcxResult<bool> {
        let _ = (
            proof_req_json,
            proof_json,
            schemas_json,
            credential_defs_json,
            rev_reg_defs_json,
            rev_regs_json,
        );

        let presentation: Presentation = serde_json::from_str(proof_json)?;
        let pres_req: PresentationRequest = serde_json::from_str(proof_req_json)?;

        let schemas: HashMap<SchemaId, Schema> = serde_json::from_str(schemas_json)?;
        let cred_defs: HashMap<CredentialDefinitionId, CredentialDefinition> =
            serde_json::from_str(credential_defs_json)?;

        let rev_reg_defs: Option<HashMap<RevocationRegistryId, RevocationRegistryDefinition>> =
            serde_json::from_str(rev_reg_defs_json)?;

        let rev_regs: Option<HashMap<RevocationRegistryId, HashMap<u64, RevocationRegistry>>> =
            serde_json::from_str(rev_regs_json)?;
        let rev_regs: Option<HashMap<RevocationRegistryId, HashMap<u64, &RevocationRegistry>>> =
            rev_regs.as_ref().map(|regs| {
                let mut new_regs: HashMap<RevocationRegistryId, HashMap<u64, &RevocationRegistry>> = HashMap::new();
                for (k, v) in regs {
                    new_regs.insert(k.clone(), hashmap_as_ref(v));
                }
                new_regs
            });

        Ok(credx::verifier::verify_presentation(
            &presentation,
            &pres_req,
            &hashmap_as_ref(&schemas),
            &hashmap_as_ref(&cred_defs),
            rev_reg_defs.as_ref().map(|regs| hashmap_as_ref(regs)).as_ref(),
            rev_regs.as_ref(),
        )?)
    }

    async fn issuer_create_and_store_revoc_reg(
        &self,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: &str,
    ) -> VcxResult<(String, String, String)> {
        let _ = (issuer_did, cred_def_id, tails_dir, max_creds, tag);
        Err(unimplemented_method_err("credx issuer_create_and_store_revoc_reg"))
    }

    async fn issuer_create_and_store_credential_def(
        &self,
        issuer_did: &str,
        schema_json: &str,
        tag: &str,
        sig_type: Option<&str>,
        config_json: &str,
    ) -> VcxResult<(String, String)> {
        let _ = (issuer_did, schema_json, tag, sig_type, config_json);
        Err(unimplemented_method_err("credx issuer_create_and_store_credential_def"))
    }

    async fn issuer_create_credential_offer(&self, cred_def_id: &str) -> VcxResult<String> {
        let _ = cred_def_id;
        Err(unimplemented_method_err("credx issuer_create_credential_offer"))
    }

    async fn issuer_create_credential(
        &self,
        cred_offer_json: &str,
        cred_req_json: &str,
        cred_values_json: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> VcxResult<(String, Option<String>, Option<String>)> {
        let _ = (cred_offer_json, cred_req_json, cred_values_json, rev_reg_id, tails_dir);
        Err(unimplemented_method_err("credx issuer_create_credential"))
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
        proof_req_json: &str,
        requested_credentials_json: &str,
        link_secret_id: &str,
        schemas_json: &str,
        credential_defs_json: &str,
        revoc_states_json: Option<&str>,
    ) -> VcxResult<String> {
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

        let schemas: HashMap<SchemaId, Schema> = serde_json::from_str(schemas_json)?;
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

            if let Some((_, _, _, req_attr_refts_revealed, _)) = proof_details_by_cred_id.get_mut(cred_id) {
                // mapping made for this credential already, add reft and its revealed status
                req_attr_refts_revealed.push((reft.to_string(), revealed));
            } else {
                let credential = self._get_credential(cred_id).await?;

                let (timestamp, rev_state) = get_rev_state(cred_id, &credential, detail, rev_states.as_ref())?;

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
                let credential = self._get_credential(cred_id).await?;

                let (timestamp, rev_state) = get_rev_state(cred_id, &credential, detail, rev_states.as_ref())?;

                proof_details_by_cred_id.insert(
                    cred_id.to_string(),
                    (credential, timestamp, rev_state, vec![], vec![reft.to_string()]),
                );
            }
        }

        // add all accumulated requested attributes and requested predicates to credx [PresentCredential]
        // object
        for (_cred_id, (credential, timestamp, rev_state, req_attr_refts_revealed, req_preds_refts)) in
            proof_details_by_cred_id.iter()
        {
            let mut add_cred = present_credentials.add_credential(credential, *timestamp, rev_state.as_ref());

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

        let link_secret = self.get_link_secret(link_secret_id).await?;

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

    async fn prover_get_credential(&self, cred_id: &str) -> VcxResult<String> {
        let cred = self._get_credential(cred_id).await?;

        let cred_info = _make_cred_info(cred_id, &cred)?;

        Ok(serde_json::to_string(&cred_info)?)
    }

    async fn prover_get_credentials(&self, filter_json: Option<&str>) -> VcxResult<String> {
        // filter_json should map to WQL query directly
        // TODO - future - may wish to validate the filter_json for more accurate error reporting

        let creds_wql = filter_json.map_or("{}", |x| x);
        let creds = self._get_credentials(creds_wql).await?;

        let cred_info_list: VcxResult<Vec<Value>> = creds
            .iter()
            .map(|(credential_id, cred)| _make_cred_info(credential_id, cred))
            .collect();

        let cred_info_list = cred_info_list?;

        Ok(serde_json::to_string(&cred_info_list)?)
    }

    async fn prover_get_credentials_for_proof_req(&self, proof_req: &str) -> VcxResult<String> {
        let proof_req_v: Value = serde_json::from_str(proof_req)
            .map_err(|e| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidProofRequest, e))?;

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

        // handle special case of "empty because json is bad" vs "empty because no attributes sepected"
        if requested_attributes.is_none() && requested_predicates.is_none() {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidAttributesStructure,
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
                .or_else(|| requested_predicates.as_ref().and_then(|req_preds| req_preds.get(&reft)))
                .ok_or(AriesVcxError::from_msg(
                    // should not happen
                    AriesVcxErrorKind::InvalidState,
                    format!("Unknown referent: {}", reft),
                ))?;

            let _attr_name = requested_val.try_get("name")?;
            let _attr_name = _attr_name.try_as_str()?;
            let attr_name = _normalize_attr_name(_attr_name);

            let non_revoked = requested_val.get("non_revoked"); // note that aca-py askar fetches from proof_req json
            let restrictions = requested_val.get("restrictions");

            let credx_creds = self
                ._get_credentials_for_proof_req_for_attr_name(restrictions, &attr_name)
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
        prover_did: &str,
        credential_offer_json: &str,
        credential_def_json: &str,
        link_secret_id: &str,
    ) -> VcxResult<(String, String)> {
        let prover_did = DidValue::new(prover_did, None);
        let cred_def: CredentialDefinition = serde_json::from_str(credential_def_json)?;
        let credential_offer: CredentialOffer = serde_json::from_str(credential_offer_json)?;
        let link_secret = self.get_link_secret(link_secret_id).await?;

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
    ) -> VcxResult<String> {
        let revoc_reg_def: RevocationRegistryDefinition = serde_json::from_str(rev_reg_def_json)?;
        let tails_file_hash = match revoc_reg_def.borrow() {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(r) => &r.value.tails_hash,
        };
        let tails_file_path = format!("{}/{}", tails_dir, tails_file_hash);
        let tails_reader: credx::tails::TailsReader = credx::tails::TailsFileReader::new(&tails_file_path);
        let rev_reg_delta: RevocationRegistryDelta = serde_json::from_str(rev_reg_delta_json)?;
        let rev_reg_idx: u32 = cred_rev_id
            .parse()
            .map_err(|e| AriesVcxError::from_msg(AriesVcxErrorKind::ParsingError, e))?;

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
        cred_id: Option<&str>,
        cred_req_meta: &str,
        cred_json: &str,
        cred_def_json: &str,
        rev_reg_def_json: Option<&str>,
    ) -> VcxResult<String> {
        let mut credential: CredxCredential = serde_json::from_str(cred_json)?;
        let cred_request_metadata: CredentialRequestMetadata = serde_json::from_str(cred_req_meta)?;
        let link_secret_id = &cred_request_metadata.master_secret_name;
        let link_secret = self.get_link_secret(link_secret_id).await?;
        let cred_def: CredentialDefinition = serde_json::from_str(cred_def_json)?;
        let rev_reg_def: Option<RevocationRegistryDefinition> = if let Some(rev_reg_def_json) = rev_reg_def_json {
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
            schema_id.parts().ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidSchema,
                "Could not process credential.schema_id as parts.",
            ))?;

        let cred_def_id = &credential.cred_def_id;
        let (_cred_def_method, issuer_did, _signature_type, _schema_id, _tag) =
            cred_def_id.parts().ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidSchema,
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

        let credential_id = cred_id.map_or(uuid(), String::from);

        let record_value = serde_json::to_string(&credential)?;
        let tags_json = serde_json::to_string(&tags)?;

        self.profile
            .inject_wallet()
            .add_wallet_record(CATEGORY_CREDENTIAL, &credential_id, &record_value, Some(&tags_json))
            .await?;

        Ok(credential_id)
    }

    async fn prover_create_link_secret(&self, link_secret_id: &str) -> VcxResult<String> {
        let wallet = self.profile.inject_wallet();

        let existing_record = wallet
            .get_wallet_record(CATEGORY_LINK_SECRET, link_secret_id, "{}")
            .await
            .ok(); // ignore error, as we only care about whether it exists or not

        if existing_record.is_some() {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::DuplicationMasterSecret,
                format!("Master secret id: {} already exists in wallet.", link_secret_id),
            ));
        }

        let secret = credx::prover::create_master_secret()?;
        let ms_decimal = secret
            .value
            .value()
            .map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::UrsaError,
                    format!(
                        "failed to get BigNumber from master secret, UrsaErrorKind: {}",
                        err.kind()
                    ),
                )
            })?
            .to_dec()
            .map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::UrsaError,
                    format!(
                        "Failed convert BigNumber to decimal string, UrsaErrorKind: {}",
                        err.kind()
                    ),
                )
            })?;

        wallet
            .add_wallet_record(CATEGORY_LINK_SECRET, link_secret_id, &ms_decimal, None)
            .await?;

        return Ok(link_secret_id.to_string());
    }

    async fn prover_delete_credential(&self, cred_id: &str) -> VcxResult<()> {
        let wallet = self.profile.inject_wallet();

        wallet.delete_wallet_record(CATEGORY_CREDENTIAL, cred_id).await
    }

    async fn issuer_create_schema(
        &self,
        issuer_did: &str,
        name: &str,
        version: &str,
        attrs: &str,
    ) -> VcxResult<(String, String)> {
        let origin_did = DidValue::new(issuer_did, None);
        let attr_names = serde_json::from_str(attrs)?;

        let schema = credx::issuer::create_schema(&origin_did, name, version, attr_names, None)?;

        let schema_json = serde_json::to_string(&schema)?;
        let schema_id = &schema.id().0;

        // TODO - future - store as cache against issuer_did
        Ok((schema_id.to_string(), schema_json))
    }

    async fn revoke_credential_local(&self, tails_dir: &str, rev_reg_id: &str, cred_rev_id: &str) -> VcxResult<()> {
        let _ = (tails_dir, rev_reg_id, cred_rev_id);
        Err(unimplemented_method_err("credx revoke_credential_local"))
    }

    async fn publish_local_revocations(&self, submitter_did: &str, rev_reg_id: &str) -> VcxResult<()> {
        let _ = (submitter_did, rev_reg_id);
        Err(unimplemented_method_err("credx publish_local_revocations"))
    }

    async fn generate_nonce(&self) -> VcxResult<String> {
        let nonce = credx::verifier::generate_nonce()?.to_string();
        Ok(nonce)
    }
}

fn get_rev_state(
    cred_id: &str,
    credential: &CredxCredential,
    detail: &Value,
    rev_states: Option<&Value>,
) -> VcxResult<(Option<u64>, Option<CredentialRevocationState>)> {
    let timestamp = detail.get("timestamp").and_then(|timestamp| timestamp.as_u64());
    let cred_rev_reg_id = credential.rev_reg_id.as_ref().map(|id| id.0.to_string());
    let rev_state = if let (Some(timestamp), Some(cred_rev_reg_id)) = (timestamp, cred_rev_reg_id) {
        let rev_state = rev_states
            .as_ref()
            .and_then(|_rev_states| _rev_states.get(cred_rev_reg_id.to_string()));
        let rev_state = rev_state.ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "No revocation states provided for credential '{}' with rev_reg_id '{}'",
                cred_id, cred_rev_reg_id
            ),
        ))?;

        let rev_state = rev_state.get(timestamp.to_string()).ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "No revocation states provided for credential '{}' with rev_reg_id '{}' at timestamp '{}'",
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

fn _make_cred_info(credential_id: &str, cred: &CredxCredential) -> VcxResult<Value> {
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

fn unimplemented_method_err(method_name: &str) -> AriesVcxError {
    AriesVcxError::from_msg(
        AriesVcxErrorKind::UnimplementedFeature,
        format!("method '{}' is not yet implemented in AriesVCX", method_name),
    )
}

// common transformation requirement in credx
fn hashmap_as_ref<'a, T, U>(map: &'a HashMap<T, U>) -> HashMap<T, &'a U>
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

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use super::IndyCredxAnonCreds;
    use crate::{
        common::test_utils::mock_profile,
        errors::error::{AriesVcxErrorKind, VcxResult},
        plugins::anoncreds::base_anoncreds::BaseAnonCreds,
    };

    #[tokio::test]
    async fn test_unimplemented_methods() {
        // test used to assert which methods are unimplemented currently, can be removed after all methods
        // implemented

        fn assert_unimplemented<T: std::fmt::Debug>(result: VcxResult<T>) {
            assert_eq!(result.unwrap_err().kind(), AriesVcxErrorKind::UnimplementedFeature)
        }

        let profile = mock_profile();
        let anoncreds: Box<dyn BaseAnonCreds> = Box::new(IndyCredxAnonCreds::new(profile));

        assert_unimplemented(anoncreds.issuer_create_and_store_revoc_reg("", "", "", 0, "").await);
        assert_unimplemented(
            anoncreds
                .issuer_create_and_store_credential_def("", "", "", None, "")
                .await,
        );
        assert_unimplemented(anoncreds.issuer_create_credential_offer("").await);
        assert_unimplemented(anoncreds.issuer_create_credential("", "", "", None, None).await);
        assert_unimplemented(anoncreds.revoke_credential_local("", "", "").await);
        assert_unimplemented(anoncreds.publish_local_revocations("", "").await);
    }
}
