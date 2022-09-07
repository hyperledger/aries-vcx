use indy::future::TryFutureExt;
use indy::{anoncreds, blob_storage, ledger};
use indy_sys::{WalletHandle, PoolHandle};
use serde_json;
use serde_json::{map::Map, Value};
use time;

use crate::error::prelude::*;
use crate::global::settings;
use crate::libindy::utils::cache::{clear_rev_reg_delta_cache, get_rev_reg_delta_cache, set_rev_reg_delta_cache};
use crate::libindy::utils::ledger::publish_txn_on_ledger;
use crate::libindy::utils::ledger::*;
use crate::libindy::utils::LibindyMock;
use crate::utils;
use crate::utils::constants::{
    rev_def_json, CRED_DEF_ID, CRED_DEF_JSON, CRED_DEF_REQ, REVOC_REG_TYPE, REV_REG_DELTA_JSON, REV_REG_ID,
    REV_REG_JSON, SCHEMA_ID, SCHEMA_JSON, SCHEMA_TXN,
};
use crate::utils::constants::{
    ATTRS, LIBINDY_CRED_OFFER, PROOF_REQUESTED_PREDICATES, REQUESTED_ATTRIBUTES, REV_STATE_JSON,
};
use crate::utils::mockdata::mock_settings::get_mock_creds_retrieved_for_proof_request;

const BLOB_STORAGE_TYPE: &str = "default";
const REVOCATION_REGISTRY_TYPE: &str = "ISSUANCE_BY_DEFAULT";

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: String,
    pub max_cred_num: u32,
    pub public_keys: serde_json::Value,
    pub tails_hash: String,
    pub tails_location: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinition {
    pub id: String,
    pub revoc_def_type: String,
    pub tag: String,
    pub cred_def_id: String,
    pub value: RevocationRegistryDefinitionValue,
    pub ver: String,
}

pub async fn libindy_verifier_verify_proof(
    proof_req_json: &str,
    proof_json: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    rev_reg_defs_json: &str,
    rev_regs_json: &str,
) -> VcxResult<bool> {
    anoncreds::verifier_verify_proof(
        proof_req_json,
        proof_json,
        schemas_json,
        credential_defs_json,
        rev_reg_defs_json,
        rev_regs_json,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_create_and_store_revoc_reg(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, String, String)> {
    trace!("creating revocation: {}, {}, {}", cred_def_id, tails_dir, max_creds);

    let tails_config = json!({"base_dir": tails_dir,"uri_pattern": ""}).to_string();

    let writer = blob_storage::open_writer(BLOB_STORAGE_TYPE, &tails_config).await?;

    let revoc_config = json!({"max_cred_num": max_creds, "issuance_type": REVOCATION_REGISTRY_TYPE}).to_string();

    anoncreds::issuer_create_and_store_revoc_reg(
        wallet_handle,
        issuer_did,
        None,
        tag,
        cred_def_id,
        &revoc_config,
        writer,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_create_and_store_credential_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    config_json: &str,
) -> VcxResult<(String, String)> {
    anoncreds::issuer_create_and_store_credential_def(
        wallet_handle,
        issuer_did,
        schema_json,
        tag,
        sig_type,
        config_json,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_issuer_create_credential_offer(
    wallet_handle: WalletHandle,
    cred_def_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        let rc = LibindyMock::get_result();
        if rc != 0 {
            return Err(VcxError::from(VcxErrorKind::InvalidState));
        };
        return Ok(LIBINDY_CRED_OFFER.to_string());
    }
    anoncreds::issuer_create_credential_offer(wallet_handle, cred_def_id)
        .await
        .map_err(VcxError::from)
}

async fn blob_storage_open_reader(base_dir: &str) -> VcxResult<i32> {
    let tails_config = json!({"base_dir": base_dir,"uri_pattern": ""}).to_string();
    blob_storage::open_reader("default", &tails_config)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_issuer_create_credential(
    wallet_handle: WalletHandle,
    cred_offer_json: &str,
    cred_req_json: &str,
    cred_values_json: &str,
    rev_reg_id: Option<String>,
    tails_file: Option<String>,
) -> VcxResult<(String, Option<String>, Option<String>)> {
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::CREDENTIAL_JSON.to_owned(), None, None));
    }

    let revocation = rev_reg_id.as_deref();

    let blob_handle = match tails_file {
        Some(x) => blob_storage_open_reader(&x).await?,
        None => -1,
    };
    anoncreds::issuer_create_credential(
        wallet_handle,
        cred_offer_json,
        cred_req_json,
        cred_values_json,
        revocation,
        blob_handle,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_prover_create_proof(
    wallet_handle: WalletHandle,
    proof_req_json: &str,
    requested_credentials_json: &str,
    master_secret_id: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    revoc_states_json: Option<&str>,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(utils::constants::PROOF_JSON.to_owned());
    }

    let revoc_states_json = revoc_states_json.unwrap_or("{}");
    anoncreds::prover_create_proof(
        wallet_handle,
        proof_req_json,
        requested_credentials_json,
        master_secret_id,
        schemas_json,
        credential_defs_json,
        revoc_states_json,
    )
    .await
    .map_err(VcxError::from)
}

async fn fetch_credentials(search_handle: i32, requested_attributes: Map<String, Value>) -> VcxResult<String> {
    let mut v: Value = json!({});
    for item_referent in requested_attributes.keys() {
        v[ATTRS][item_referent] = serde_json::from_str(
            &anoncreds::prover_fetch_credentials_for_proof_req(search_handle, item_referent, 100).await?,
        )
        .map_err(|_| {
            error!("Invalid Json Parsing of Object Returned from Libindy. Did Libindy change its structure?");
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                "Invalid Json Parsing of Object Returned from Libindy. Did Libindy change its structure?",
            )
        })?
    }

    Ok(v.to_string())
}

async fn close_search_handle(search_handle: i32) -> VcxResult<()> {
    anoncreds::prover_close_credentials_search_for_proof_req(search_handle)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_get_credentials_for_proof_req(
    wallet_handle: WalletHandle,
    proof_req: &str,
) -> VcxResult<String> {
    trace!(
        "libindy_prover_get_credentials_for_proof_req >>> proof_req: {}",
        proof_req
    );
    match get_mock_creds_retrieved_for_proof_request() {
        None => {}
        Some(mocked_creds) => {
            warn!("get_mock_creds_retrieved_for_proof_request  returning mocked response");
            return Ok(mocked_creds);
        }
    }

    // this may be too redundant since Prover::search_credentials will validate the proof reqeuest already.
    let proof_request_json: Map<String, Value> = serde_json::from_str(proof_req).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidProofRequest,
            format!("Cannot deserialize ProofRequest: {:?}", err),
        )
    })?;

    // since the search_credentials_for_proof request validates that the proof_req is properly structured, this get()
    // fn should never fail, unless libindy changes their formats.
    let requested_attributes: Option<Map<String, Value>> = proof_request_json.get(REQUESTED_ATTRIBUTES).and_then(|v| {
        serde_json::from_value(v.clone())
            .map_err(|_| {
                error!("Invalid Json Parsing of Requested Attributes Retrieved From Libindy. Did Libindy change its structure?");
            })
            .ok()
    });

    let requested_predicates: Option<Map<String, Value>> = proof_request_json.get(PROOF_REQUESTED_PREDICATES).and_then(|v| {
        serde_json::from_value(v.clone())
            .map_err(|_| {
                error!("Invalid Json Parsing of Requested Predicates Retrieved From Libindy. Did Libindy change its structure?");
            })
            .ok()
    });

    // handle special case of "empty because json is bad" vs "empty because no attributes sepected"
    if requested_attributes == None && requested_predicates == None {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidAttributesStructure,
            "Invalid Json Parsing of Requested Attributes Retrieved From Libindy",
        ));
    }

    let mut fetch_attrs: Map<String, Value> = match requested_attributes {
        Some(attrs) => attrs.clone(),
        None => Map::new(),
    };
    match requested_predicates {
        Some(attrs) => fetch_attrs.extend(attrs),
        None => (),
    }
    if !fetch_attrs.is_empty() {
        let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, proof_req, None)
            .await
            .map_err(|ec| {
                error!("Opening Indy Search for Credentials Failed");
                ec
            })?;
        let creds: String = fetch_credentials(search_handle, fetch_attrs).await?;

        // should an error on closing a search handle throw an error, or just a warning?
        // for now we're are just outputting to the user that there is an issue, and continuing on.
        let _ = close_search_handle(search_handle);
        Ok(creds)
    } else {
        Ok("{}".to_string())
    }
}

pub async fn libindy_prover_create_credential_req(
    wallet_handle: WalletHandle,
    prover_did: &str,
    credential_offer_json: &str,
    credential_def_json: &str,
) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new()));
    }

    let master_secret_name = settings::DEFAULT_LINK_SECRET_ALIAS;
    anoncreds::prover_create_credential_req(
        wallet_handle,
        prover_did,
        credential_offer_json,
        credential_def_json,
        master_secret_name,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_prover_create_revocation_state(
    rev_reg_def_json: &str,
    rev_reg_delta_json: &str,
    cred_rev_id: &str,
    tails_file: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_STATE_JSON.to_string());
    }

    let blob_handle = blob_storage_open_reader(tails_file).await?;

    anoncreds::create_revocation_state(blob_handle, rev_reg_def_json, rev_reg_delta_json, 100, cred_rev_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_update_revocation_state(
    rev_reg_def_json: &str,
    rev_state_json: &str,
    rev_reg_delta_json: &str,
    cred_rev_id: &str,
    tails_file: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_STATE_JSON.to_string());
    }

    let blob_handle = blob_storage_open_reader(tails_file).await?;

    anoncreds::update_revocation_state(
        blob_handle,
        rev_state_json,
        rev_reg_def_json,
        rev_reg_delta_json,
        100,
        cred_rev_id,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_prover_store_credential(
    wallet_handle: WalletHandle,
    cred_id: Option<&str>,
    cred_req_meta: &str,
    cred_json: &str,
    cred_def_json: &str,
    rev_reg_def_json: Option<&str>,
) -> VcxResult<String> {
    trace!("libindy_prover_store_credential >>> cred_id: {:?}, cred_req_meta: {}, cred_json: {}, cred_def_json: {}, rev_reg_def_json: {:?}", cred_id, cred_req_meta, cred_json, cred_def_json, rev_reg_def_json);
    if settings::indy_mocks_enabled() {
        return Ok("cred_id".to_string());
    }

    anoncreds::prover_store_credential(
        wallet_handle,
        cred_id,
        cred_req_meta,
        cred_json,
        cred_def_json,
        rev_reg_def_json,
    )
    .await
    .map_err(VcxError::from)
}

pub async fn libindy_prover_delete_credential(wallet_handle: WalletHandle, cred_id: &str) -> VcxResult<()> {
    anoncreds::prover_delete_credential(wallet_handle, cred_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_create_master_secret(
    wallet_handle: WalletHandle,
    master_secret_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string());
    }

    anoncreds::prover_create_master_secret(wallet_handle, Some(master_secret_id))
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_issuer_create_schema(
    issuer_did: &str,
    name: &str,
    version: &str,
    attrs: &str,
) -> VcxResult<(String, String)> {
    trace!(
        "libindy_issuer_create_schema >>> issuer_did: {}, name: {}, version: {}, attrs: {}",
        issuer_did,
        name,
        version,
        attrs
    );

    anoncreds::issuer_create_schema(issuer_did, name, version, attrs)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_issuer_revoke_credential(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<String> {
    let blob_handle = blob_storage_open_reader(tails_file).await?;

    anoncreds::issuer_revoke_credential(wallet_handle, blob_handle, rev_reg_id, cred_rev_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_issuer_merge_revocation_registry_deltas(old_delta: &str, new_delta: &str) -> VcxResult<String> {
    anoncreds::issuer_merge_revocation_registry_deltas(old_delta, new_delta)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_build_revoc_reg_def_request(submitter_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok("".to_string());
    }

    ledger::build_revoc_reg_def_request(submitter_did, rev_reg_def_json)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_build_revoc_reg_entry_request(
    submitter_did: &str,
    rev_reg_id: &str,
    rev_def_type: &str,
    value: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok("".to_string());
    }

    ledger::build_revoc_reg_entry_request(submitter_did, rev_reg_id, rev_def_type, value)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_build_get_revoc_reg_def_request(submitter_did: &str, rev_reg_id: &str) -> VcxResult<String> {
    ledger::build_get_revoc_reg_def_request(Some(submitter_did), rev_reg_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_parse_get_revoc_reg_def_response(rev_reg_def_json: &str) -> VcxResult<(String, String)> {
    ledger::parse_get_revoc_reg_def_response(rev_reg_def_json)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_build_get_revoc_reg_delta_request(
    submitter_did: &str,
    rev_reg_id: &str,
    from: i64,
    to: i64,
) -> VcxResult<String> {
    ledger::build_get_revoc_reg_delta_request(Some(submitter_did), rev_reg_id, from, to)
        .await
        .map_err(VcxError::from)
}

async fn libindy_build_get_revoc_reg_request(
    submitter_did: &str,
    rev_reg_id: &str,
    timestamp: u64,
) -> VcxResult<String> {
    ledger::build_get_revoc_reg_request(Some(submitter_did), rev_reg_id, timestamp as i64)
        .await
        .map_err(VcxError::from)
}

async fn libindy_parse_get_revoc_reg_response(get_cred_def_resp: &str) -> VcxResult<(String, String, u64)> {
    ledger::parse_get_revoc_reg_response(get_cred_def_resp)
        .await
        .map_err(VcxError::from)
}

async fn libindy_parse_get_cred_def_response(get_rev_reg_resp: &str) -> VcxResult<(String, String)> {
    ledger::parse_get_cred_def_response(get_rev_reg_resp)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_parse_get_revoc_reg_delta_response(
    get_rev_reg_delta_response: &str,
) -> VcxResult<(String, String, u64)> {
    ledger::parse_get_revoc_reg_delta_response(get_rev_reg_delta_response)
        .await
        .map_err(VcxError::from)
}

pub async fn create_schema(submitter_did: &str, name: &str, version: &str, data: &str) -> VcxResult<(String, String)> {
    trace!("create_schema >>> submitter_did: {}, name: {}, version: {}, data: {}", submitter_did, name, version, data);

    if settings::indy_mocks_enabled() {
        return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
    }

    let (id, create_schema) = libindy_issuer_create_schema(&submitter_did, name, version, data).await?;

    Ok((id, create_schema))
}

pub async fn build_schema_request(submitter_did: &str, schema: &str) -> VcxResult<String> {
    trace!("build_schema_request >>> submitter_did: {}, schema: {}", submitter_did, schema);

    if settings::indy_mocks_enabled() {
        return Ok(SCHEMA_TXN.to_string());
    }

    let request = libindy_build_schema_request(&submitter_did, schema).await?;

    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
}

pub async fn publish_schema(submitter_did: &str, wallet_handle: WalletHandle, schema: &str) -> VcxResult<()> {
    trace!("publish_schema >>> submitter_did: {}, schema: {}", submitter_did, schema);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    let request = build_schema_request(submitter_did, schema).await?;

    let response = publish_txn_on_ledger(wallet_handle, submitter_did, &request).await?;

    _check_schema_response(&response)?;

    Ok(())
}

pub async fn get_schema_json(wallet_handle: WalletHandle, pool_handle: PoolHandle, schema_id: &str) -> VcxResult<(String, String)> {
    trace!("get_schema_json >>> schema_id: {}", schema_id);
    if settings::indy_mocks_enabled() {
        return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
    }

    let submitter_did = crate::utils::random::generate_random_did();

    let schema_json = libindy_get_schema(wallet_handle, pool_handle, &submitter_did, schema_id).await?;

    Ok((schema_id.to_string(), schema_json))
}

pub async fn generate_cred_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    support_revocation: Option<bool>,
) -> VcxResult<(String, String)> {
    trace!(
        "generate_cred_def >>> issuer_did: {}, schema_json: {}, tag: {}, sig_type: {:?}, support_revocation: {:?}",
        issuer_did,
        schema_json,
        tag,
        sig_type,
        support_revocation
    );
    if settings::indy_mocks_enabled() {
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let config_json = json!({"support_revocation": support_revocation.unwrap_or(false)}).to_string();

    libindy_create_and_store_credential_def(wallet_handle, issuer_did, schema_json, tag, sig_type, &config_json).await
}

pub async fn build_cred_def_request(issuer_did: &str, cred_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(CRED_DEF_REQ.to_string());
    }

    let cred_def_req = libindy_build_create_credential_def_txn(issuer_did, cred_def_json).await?;

    let cred_def_req = append_txn_author_agreement_to_request(&cred_def_req).await?;

    Ok(cred_def_req)
}

pub async fn publish_cred_def(wallet_handle: WalletHandle, issuer_did: &str, cred_def_json: &str) -> VcxResult<()> {
    trace!(
        "publish_cred_def >>> issuer_did: {}, cred_def_json: {}",
        issuer_did,
        cred_def_json
    );
    if settings::indy_mocks_enabled() {
        debug!("publish_cred_def >>> mocked success");
        return Ok(());
    }
    let cred_def_req = build_cred_def_request(issuer_did, cred_def_json).await?;
    publish_txn_on_ledger(wallet_handle, issuer_did, &cred_def_req).await?;
    Ok(())
}

pub async fn get_cred_def_json(wallet_handle: WalletHandle, pool_handle: PoolHandle, cred_def_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        debug!("get_cred_def_json >>> returning mocked value");
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let cred_def_json = libindy_get_cred_def(wallet_handle, pool_handle, cred_def_id).await?;

    Ok((cred_def_id.to_string(), cred_def_json))
}

pub async fn generate_rev_reg(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, RevocationRegistryDefinition, String)> {
    trace!(
        "generate_rev_reg >>> issuer_did: {}, cred_def_id: {}, tails_file: {}, max_creds: {}, tag: {}",
        issuer_did,
        cred_def_id,
        tails_dir,
        max_creds,
        tag
    );
    if settings::indy_mocks_enabled() {
        debug!("generate_rev_reg >>> returning mocked value");
        return Ok((
            REV_REG_ID.to_string(),
            RevocationRegistryDefinition::default(),
            "".to_string(),
        ));
    }

    let (rev_reg_id, rev_reg_def_json, rev_reg_entry_json) =
        libindy_create_and_store_revoc_reg(wallet_handle, issuer_did, cred_def_id, tails_dir, max_creds, tag).await?;

    let rev_reg_def: RevocationRegistryDefinition = serde_json::from_str(&rev_reg_def_json).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!(
                "Failed to deserialize rev_reg_def: {:?}, error: {:?}",
                rev_reg_def_json, err
            ),
        )
    })?;

    Ok((rev_reg_id, rev_reg_def, rev_reg_entry_json))
}

pub async fn build_rev_reg_request(issuer_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        debug!("build_rev_reg_request >>> returning mocked value");
        return Ok("".to_string());
    }

    let rev_reg_def_req = libindy_build_revoc_reg_def_request(issuer_did, rev_reg_def_json).await?;
    let rev_reg_def_req = append_txn_author_agreement_to_request(&rev_reg_def_req).await?;
    Ok(rev_reg_def_req)
}

pub async fn publish_rev_reg_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    rev_reg_def: &RevocationRegistryDefinition,
) -> VcxResult<()> {
    trace!("publish_rev_reg_def >>> issuer_did: {}, rev_reg_def: ...", issuer_did);
    if settings::indy_mocks_enabled() {
        debug!("publish_rev_reg_def >>> mocked success");
        return Ok(());
    }

    let rev_reg_def_json = serde_json::to_string(&rev_reg_def).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to serialize rev_reg_def: {:?}, error: {:?}", rev_reg_def, err),
        )
    })?;
    let rev_reg_def_req = build_rev_reg_request(issuer_did, &rev_reg_def_json).await?;
    publish_txn_on_ledger(wallet_handle, issuer_did, &rev_reg_def_req).await?;
    Ok(())
}

pub async fn get_rev_reg_def_json(pool_handle: PoolHandle, rev_reg_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        debug!("get_rev_reg_def_json >>> returning mocked value");
        return Ok((REV_REG_ID.to_string(), rev_def_json()));
    }

    let submitter_did = crate::utils::random::generate_random_did();

    libindy_build_get_revoc_reg_def_request(&submitter_did, rev_reg_id)
        .and_then(|req| async move { libindy_submit_request(pool_handle, &req).await })
        .and_then(|response| async move { libindy_parse_get_revoc_reg_def_response(&response).await })
        .await
}

pub async fn build_rev_reg_delta_request(
    issuer_did: &str,
    rev_reg_id: &str,
    rev_reg_entry_json: &str,
) -> VcxResult<String> {
    trace!(
        "build_rev_reg_delta_request >>> issuer_did: {}, rev_reg_id: {}, rev_reg_entry_json: {}",
        issuer_did,
        rev_reg_id,
        rev_reg_entry_json
    );
    let request =
        libindy_build_revoc_reg_entry_request(issuer_did, rev_reg_id, REVOC_REG_TYPE, rev_reg_entry_json).await?;
    let request = append_txn_author_agreement_to_request(&request).await?;
    Ok(request)
}

pub async fn publish_rev_reg_delta(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    rev_reg_id: &str,
    rev_reg_entry_json: &str,
) -> VcxResult<String> {
    trace!(
        "publish_rev_reg_delta >>> issuer_did: {}, rev_reg_id: {}, rev_reg_entry_json: {}",
        issuer_did,
        rev_reg_id,
        rev_reg_entry_json
    );
    let request = build_rev_reg_delta_request(issuer_did, rev_reg_id, rev_reg_entry_json).await?;
    publish_txn_on_ledger(wallet_handle, issuer_did, &request).await
}

pub async fn get_rev_reg_delta_json(
    pool_handle: PoolHandle,
    rev_reg_id: &str,
    from: Option<u64>,
    to: Option<u64>,
) -> VcxResult<(String, String, u64)> {
    trace!(
        "get_rev_reg_delta_json >>> pool_handle: {:?}, rev_reg_id: {}, from: {:?}, to: {:?}",
        pool_handle,
        rev_reg_id,
        from,
        to
    );
    if settings::indy_mocks_enabled() {
        debug!("get_rev_reg_delta_json >>> returning mocked value");
        return Ok((REV_REG_ID.to_string(), REV_REG_DELTA_JSON.to_string(), 1));
    }

    let submitter_did = crate::utils::random::generate_random_did();

    let from: i64 = if let Some(_from) = from { _from as i64 } else { -1 };
    let to = if let Some(_to) = to {
        _to as i64
    } else {
        time::get_time().sec
    };

    libindy_build_get_revoc_reg_delta_request(&submitter_did, rev_reg_id, from, to)
        .and_then(|req| async move { libindy_submit_request(pool_handle, &req).await })
        .and_then(|response| async move { libindy_parse_get_revoc_reg_delta_response(&response).await })
        .await
}

pub async fn get_rev_reg(rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
    if settings::indy_mocks_enabled() {
        return Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1));
    }

    let submitter_did = crate::utils::random::generate_random_did();
    let pool_handle = crate::global::pool::get_main_pool_handle()?;

    libindy_build_get_revoc_reg_request(&submitter_did, rev_reg_id, timestamp)
        .and_then(|req| async move { libindy_submit_request(pool_handle, &req).await })
        .and_then(|response| async move { libindy_parse_get_revoc_reg_response(&response).await })
        .await
}

pub async fn get_cred_def(pool_handle: PoolHandle, issuer_did: Option<&str>, cred_def_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Err(VcxError::from(VcxErrorKind::LibndyError(309)));
    }
    libindy_build_get_cred_def_request(issuer_did, cred_def_id)
        .and_then(|req| async move { libindy_submit_request(pool_handle, &req).await })
        .and_then(|response| async move { libindy_parse_get_cred_def_response(&response).await })
        .await
}

pub async fn is_cred_def_on_ledger(pool_handle: PoolHandle, issuer_did: Option<&str>, cred_def_id: &str) -> VcxResult<bool> {
    match get_cred_def(pool_handle, issuer_did, cred_def_id).await {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == VcxErrorKind::LibndyError(309) => Ok(false),
        Err(err) => Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!(
                "Failed to check presence of credential definition id {} on the ledger\nError: {}",
                cred_def_id, err
            ),
        )),
    }
}

pub async fn revoke_credential(
    wallet_handle: WalletHandle,
    submitter_did: &str,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_REG_DELTA_JSON.to_string());
    }

    let delta = libindy_issuer_revoke_credential(wallet_handle, tails_file, rev_reg_id, cred_rev_id).await?;
    publish_rev_reg_delta(wallet_handle, &submitter_did, rev_reg_id, &delta).await?;

    Ok(delta)
}

pub async fn revoke_credential_local(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<()> {
    let mut new_delta = libindy_issuer_revoke_credential(wallet_handle, tails_file, rev_reg_id, cred_rev_id).await?;
    if let Some(old_delta) = get_rev_reg_delta_cache(wallet_handle, rev_reg_id).await {
        new_delta = libindy_issuer_merge_revocation_registry_deltas(old_delta.as_str(), new_delta.as_str()).await?;
    }
    set_rev_reg_delta_cache(wallet_handle, rev_reg_id, &new_delta).await
}

pub async fn publish_local_revocations(wallet_handle: WalletHandle, submitter_did: &str, rev_reg_id: &str) -> VcxResult<String> {
    if let Some(delta) = get_rev_reg_delta_cache(wallet_handle, rev_reg_id).await {
        match clear_rev_reg_delta_cache(wallet_handle, rev_reg_id).await {
            Ok(_) => publish_rev_reg_delta(wallet_handle, &submitter_did, rev_reg_id, &delta).await,
            Err(err) => Err(err),
        }
    } else {
        Err(VcxError::from(VcxErrorKind::RevDeltaNotFound))
    }
}

pub async fn libindy_to_unqualified(entity: &str) -> VcxResult<String> {
    anoncreds::to_unqualified(entity).await.map_err(VcxError::from)
}

async fn libindy_build_get_txn_request(submitter_did: Option<&str>, seq_no: i32) -> VcxResult<String> {
    ledger::build_get_txn_request(submitter_did, None, seq_no)
        .await
        .map_err(VcxError::from)
}

pub async fn build_get_txn_request(submitter_did: Option<&str>, seq_no: i32) -> VcxResult<String> {
    trace!(
        "build_get_txn_request >>> submitter_did: {:?}, seq_no: {}",
        submitter_did,
        seq_no
    );
    let request = libindy_build_get_txn_request(submitter_did, seq_no).await?;
    let request = append_txn_author_agreement_to_request(&request).await?;
    Ok(request)
}

pub async fn get_ledger_txn(
    wallet_handle: WalletHandle,
    submitter_did: Option<&str>,
    seq_no: i32,
) -> VcxResult<String> {
    trace!(
        "get_ledger_txn >>> submitter_did: {:?}, seq_no: {}",
        submitter_did,
        seq_no
    );
    let pool_handle = crate::global::pool::get_main_pool_handle()?;
    let req = build_get_txn_request(submitter_did, seq_no).await?;
    let res = if let Some(submitter_did) = submitter_did {
        libindy_sign_and_submit_request(wallet_handle, pool_handle, submitter_did, &req).await?
    } else {
        libindy_submit_request(pool_handle, &req).await?
    };
    _check_response(&res)?;
    Ok(res)
}

fn _check_schema_response(response: &str) -> VcxResult<()> {
    // TODO: saved backwardcampatibilyty but actually we can better handle response
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(reject) => Err(VcxError::from_msg(
            VcxErrorKind::DuplicationSchema,
            format!("{:?}", reject),
        )),
        Response::ReqNACK(reqnack) => Err(VcxError::from_msg(
            VcxErrorKind::UnknownSchemaRejection,
            format!("{:?}", reqnack),
        )),
    }
}

fn _check_response(response: &str) -> VcxResult<()> {
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!("{:?}", res),
        )),
    }
}

pub async fn generate_nonce() -> VcxResult<String> {
    anoncreds::generate_nonce().await.map_err(VcxError::from)
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::thread;
    use std::time::Duration;

    use crate::global::settings;
    use crate::libindy;
    use crate::libindy::credential_def::revocation_registry::RevocationRegistry;
    use crate::libindy::credential_def::{CredentialDef, CredentialDefConfigBuilder};
    use crate::libindy::credentials::encode_attributes;
    use crate::utils::constants::{TAILS_DIR, TEST_TAILS_URL};
    use crate::utils::get_temp_dir_path;

    use super::*;

    extern crate serde_json;

    pub async fn create_schema(attr_list: &str, submitter_did: &str) -> (String, String) {
        let data = attr_list.to_string();
        let schema_name: String = crate::utils::random::generate_random_schema_name();
        let schema_version: String = crate::utils::random::generate_random_schema_version();

        libindy_issuer_create_schema(&submitter_did, &schema_name, &schema_version, &data)
            .await
            .unwrap()
    }

    pub async fn create_schema_req(schema_json: &str, submitter_did: &str) -> String {
        let request = libindy_build_schema_request(submitter_did, schema_json)
            .await
            .unwrap();
        append_txn_author_agreement_to_request(&request).await.unwrap()
    }

    pub async fn create_and_write_test_schema(wallet_handle: WalletHandle, submitter_did: &str, attr_list: &str) -> (String, String) {
        let (schema_id, schema_json) = create_schema(attr_list, submitter_did).await;
        let req = create_schema_req(&schema_json, submitter_did).await;
        publish_txn_on_ledger(wallet_handle, submitter_did, &req).await.unwrap();
        thread::sleep(Duration::from_millis(1000));
        (schema_id, schema_json)
    }

    pub async fn create_and_store_nonrevocable_credential_def(
        wallet_handle: WalletHandle,
        issuer_did: &str,
        attr_list: &str,
    ) -> (String, String, String, String, CredentialDef) {
        let (schema_id, schema_json) = create_and_write_test_schema(wallet_handle, issuer_did, attr_list).await;
        let config = CredentialDefConfigBuilder::default()
            .issuer_did(issuer_did)
            .schema_id(&schema_id)
            .tag("1")
            .build()
            .unwrap();
        let pool_handle = crate::global::pool::get_main_pool_handle().unwrap();
        let cred_def = CredentialDef::create(wallet_handle, pool_handle, "1".to_string(), config, false)
            .await
            .unwrap()
            .publish_cred_def(wallet_handle)
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(1000));
        let cred_def_id = cred_def.get_cred_def_id();
        thread::sleep(Duration::from_millis(1000));
        let pool_handle = crate::global::pool::get_main_pool_handle().unwrap();
        let (_, cred_def_json) = get_cred_def_json(wallet_handle, pool_handle, &cred_def_id).await.unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, cred_def)
    }

    pub async fn create_and_store_credential_def(
        wallet_handle: WalletHandle,
        issuer_did: &str,
        attr_list: &str,
    ) -> (
        String,
        String,
        String,
        String,
        String,
        CredentialDef,
        RevocationRegistry,
    ) {
        let (schema_id, schema_json) = create_and_write_test_schema(wallet_handle, issuer_did, attr_list).await;
        thread::sleep(Duration::from_millis(500));
        let config = CredentialDefConfigBuilder::default()
            .issuer_did(issuer_did)
            .schema_id(&schema_id)
            .tag("1")
            .build()
            .unwrap();
        let pool_handle = crate::global::pool::get_main_pool_handle().unwrap();
        let cred_def = CredentialDef::create(wallet_handle, pool_handle, "1".to_string(), config, true)
            .await
            .unwrap()
            .publish_cred_def(wallet_handle)
            .await
            .unwrap();
        let mut rev_reg = RevocationRegistry::create(
            wallet_handle,
            issuer_did,
            &cred_def.cred_def_id,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            10,
            1,
        )
        .await
        .unwrap();
        rev_reg
            .publish_revocation_primitives(wallet_handle, TEST_TAILS_URL)
            .await
            .unwrap();

        thread::sleep(Duration::from_millis(1000));
        let cred_def_id = cred_def.get_cred_def_id();
        thread::sleep(Duration::from_millis(1000));
        let pool_handle = crate::global::pool::get_main_pool_handle().unwrap();
        let (_, cred_def_json) = get_cred_def_json(wallet_handle, pool_handle, &cred_def_id).await.unwrap();
        (
            schema_id,
            schema_json,
            cred_def_id,
            cred_def_json,
            rev_reg.get_rev_reg_id(),
            cred_def,
            rev_reg,
        )
    }

    pub async fn create_credential_req(
        wallet_handle: WalletHandle,
        did: &str,
        cred_def_id: &str,
        cred_def_json: &str,
    ) -> (String, String, String) {
        let offer = libindy::utils::anoncreds::libindy_issuer_create_credential_offer(wallet_handle, cred_def_id)
            .await
            .unwrap();
        let (req, req_meta) = libindy::utils::anoncreds::libindy_prover_create_credential_req(
            wallet_handle,
            &did,
            &offer,
            cred_def_json,
        )
        .await
        .unwrap();
        (offer, req, req_meta)
    }

    // todo: extract create_and_store_credential_def into caller functions
    pub async fn create_and_store_credential(
        wallet_handle: WalletHandle,
        institution_did: &str,
        attr_list: &str,
    ) -> (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, _, _) =
            create_and_store_credential_def(wallet_handle, institution_did, attr_list).await;

        let (offer, req, req_meta) = create_credential_req(wallet_handle, institution_did, &cred_def_id, &cred_def_json).await;

        /* create cred */
        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let encoded_attributes = encode_attributes(&credential_data).unwrap();
        let pool_handle = crate::global::pool::get_main_pool_handle().unwrap();
        let (_id, rev_def_json) = get_rev_reg_def_json(pool_handle, &rev_reg_id).await.unwrap();
        let tails_file = get_temp_dir_path(TAILS_DIR).to_str().unwrap().to_string();

        let (cred, cred_rev_id, _) = libindy::utils::anoncreds::libindy_issuer_create_credential(
            wallet_handle,
            &offer,
            &req,
            &encoded_attributes,
            Some(rev_reg_id.clone()),
            Some(tails_file),
        )
        .await
        .unwrap();
        /* store cred */
        let cred_id = libindy::utils::anoncreds::libindy_prover_store_credential(
            wallet_handle,
            None,
            &req_meta,
            &cred,
            &cred_def_json,
            Some(&rev_def_json),
        )
        .await
        .unwrap();
        (
            schema_id,
            schema_json,
            cred_def_id,
            cred_def_json,
            offer,
            req,
            req_meta,
            cred_id,
            rev_reg_id,
            cred_rev_id.unwrap(),
        )
    }

    // todo: extract create_and_store_nonrevocable_credential_def into caller functions
    pub async fn create_and_store_nonrevocable_credential(
        wallet_handle: WalletHandle,
        issuer_did: &str,
        attr_list: &str,
    ) -> (String, String, String, String, String, String, String, String) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, _) =
            create_and_store_nonrevocable_credential_def(wallet_handle, issuer_did, attr_list).await;

        let (offer, req, req_meta) = create_credential_req(wallet_handle, issuer_did, &cred_def_id, &cred_def_json).await;

        /* create cred */
        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let encoded_attributes = encode_attributes(&credential_data).unwrap();

        let (cred, _, _) = libindy::utils::anoncreds::libindy_issuer_create_credential(
            wallet_handle,
            &offer,
            &req,
            &encoded_attributes,
            None,
            None,
        )
        .await
        .unwrap();
        /* store cred */
        let cred_id = libindy::utils::anoncreds::libindy_prover_store_credential(
            wallet_handle,
            None,
            &req_meta,
            &cred,
            &cred_def_json,
            None,
        )
        .await
        .unwrap();
        (
            schema_id,
            schema_json,
            cred_def_id,
            cred_def_json,
            offer,
            req,
            req_meta,
            cred_id,
        )
    }

    pub async fn create_indy_proof(wallet_handle: WalletHandle, did: &str) -> (String, String, String, String) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
            create_and_store_nonrevocable_credential(wallet_handle, &did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;
        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "zip_2": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({}),
        })
        .to_string();
        let requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true},
                 "zip_2": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{}
        })
        .to_string();

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        })
        .to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        })
        .to_string();

        libindy_prover_get_credentials_for_proof_req(wallet_handle, &proof_req)
            .await
            .unwrap();

        let proof = libindy_prover_create_proof(
            wallet_handle,
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None,
        )
        .await
        .unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    pub async fn create_proof_with_predicate(
        wallet_handle: WalletHandle,
        did: &str,
        include_predicate_cred: bool,
    ) -> (String, String, String, String) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id) =
            create_and_store_nonrevocable_credential(wallet_handle, &did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({
               "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
           }),
        })
        .to_string();

        let requested_credentials_json;
        if include_predicate_cred {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
                  "zip_3": {"cred_id": cred_id}
              }
            })
            .to_string();
        } else {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
              }
            })
            .to_string();
        }

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        })
        .to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        })
        .to_string();

        libindy_prover_get_credentials_for_proof_req(wallet_handle, &proof_req)
            .await
            .unwrap();

        let proof = libindy_prover_create_proof(
            wallet_handle,
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None,
        )
        .await
        .unwrap();
        (schemas, cred_defs, proof_req, proof)
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use indy_sys::{WalletHandle, PoolHandle};

    use crate::libindy::utils::anoncreds::get_schema_json;
    use crate::utils::constants::{SCHEMA_ID, SCHEMA_JSON};
    use crate::utils::devsetup::SetupMocks;

    #[tokio::test]
    async fn from_ledger_schema_id() {
        let _setup = SetupMocks::init();
        let (id, retrieved_schema) = get_schema_json(WalletHandle(0), 1, SCHEMA_ID).await.unwrap();
        assert_eq!(&retrieved_schema, SCHEMA_JSON);
        assert_eq!(&id, SCHEMA_ID);
    }
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::libindy::utils::anoncreds::test_utils::{
        create_and_store_credential, create_and_store_credential_def, create_and_store_nonrevocable_credential_def,
        create_and_write_test_schema, create_indy_proof, create_proof_with_predicate,
    };
    use crate::utils::constants::TAILS_DIR;
    use crate::utils::devsetup::{SetupLibraryWallet, SetupWalletPool};
    use crate::utils::get_temp_dir_path;

    use super::*;

    extern crate serde_json;

    #[tokio::test]
    async fn test_prover_verify_proof() {
        let setup = SetupWalletPool::init().await;

        let (schemas, cred_defs, proof_req, proof) = create_indy_proof(setup.wallet_handle, &setup.institution_did).await;

        let proof_validation = libindy_verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
            .await
            .unwrap();

        assert!(proof_validation);
    }

    #[tokio::test]
    async fn test_prover_verify_proof_with_predicate_success_case() {
        let setup = SetupWalletPool::init().await;

        let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(setup.wallet_handle, &setup.institution_did, true).await;

        let proof_validation = libindy_verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
            .await
            .unwrap();

        assert!(proof_validation);
    }

    #[tokio::test]
    async fn test_prover_verify_proof_with_predicate_fail_case() {
        let setup = SetupWalletPool::init().await;

        let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(setup.wallet_handle, &setup.institution_did, false).await;

        libindy_verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
            .await
            .unwrap_err();
    }

    #[tokio::test]
    async fn tests_libindy_prover_get_credentials() {
        let setup = SetupLibraryWallet::init().await;

        let proof_req = "{";
        let result = libindy_prover_get_credentials_for_proof_req(setup.wallet_handle, &proof_req).await;
        assert_eq!(result.unwrap_err().kind(), VcxErrorKind::InvalidProofRequest);

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        })
        .to_string();
        let _result = libindy_prover_get_credentials_for_proof_req(setup.wallet_handle, &proof_req)
            .await
            .unwrap();

        let result_malformed_json = libindy_prover_get_credentials_for_proof_req(setup.wallet_handle, "{}")
            .await
            .unwrap_err();
        assert_eq!(result_malformed_json.kind(), VcxErrorKind::InvalidAttributesStructure);
    }

    #[tokio::test]
    async fn test_issuer_revoke_credential() {
        let setup = SetupWalletPool::init().await;

        let rc = libindy_issuer_revoke_credential(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            "",
            "",
        )
        .await;
        assert!(rc.is_err());

        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id) =
            create_and_store_credential(setup.wallet_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;
        let rc = libindy_issuer_revoke_credential(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            &rev_reg_id,
            &cred_rev_id,
        )
        .await;

        assert!(rc.is_ok());
    }

    #[tokio::test]
    async fn test_create_cred_def_real() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) =
            create_and_write_test_schema(setup.wallet_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;
        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await.unwrap();

        let (_, cred_def_json) = generate_cred_def(setup.wallet_handle, &setup.institution_did, &schema_json, "tag_1", None, Some(true))
            .await
            .unwrap();
        publish_cred_def(setup.wallet_handle, &setup.institution_did, &cred_def_json)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_rev_reg_def_fails_for_cred_def_created_without_revocation() {
        // todo: does not need agency setup
        let setup = SetupWalletPool::init().await;

        // Cred def is created with support_revocation=false,
        // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
        let (_, _, cred_def_id, _, _) =
            create_and_store_nonrevocable_credential_def(setup.wallet_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS)
                .await;
        let rc = generate_rev_reg(
            setup.wallet_handle,
            &setup.institution_did,
            &cred_def_id,
            get_temp_dir_path("path.txt").to_str().unwrap(),
            2,
            "tag1",
        )
        .await;

        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::LibindyInvalidStructure);
    }

    #[tokio::test]
    async fn test_create_rev_reg_def() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) =
            create_and_write_test_schema(setup.wallet_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;
        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await.unwrap();

        let (cred_def_id, cred_def_json) =
            generate_cred_def(setup.wallet_handle, &setup.institution_did, &schema_json, "tag_1", None, Some(true))
                .await
                .unwrap();
        publish_cred_def(setup.wallet_handle, &setup.institution_did, &cred_def_json)
            .await
            .unwrap();
        let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) =
            generate_rev_reg(setup.wallet_handle, &setup.institution_did, &cred_def_id, "tails.txt", 2, "tag1")
                .await
                .unwrap();
        publish_rev_reg_def(setup.wallet_handle, &setup.institution_did, &rev_reg_def_json)
            .await
            .unwrap();
        publish_rev_reg_delta(setup.wallet_handle, &setup.institution_did, &rev_reg_def_id, &rev_reg_entry_json)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_rev_reg_def_json() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) = create_and_store_credential_def(setup.wallet_handle, &setup.institution_did, attrs).await;

        let (id, _json) = get_rev_reg_def_json(setup.pool_handle, &rev_reg_id).await.unwrap();
        assert_eq!(id, rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_rev_reg_delta_json() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) = create_and_store_credential_def(setup.wallet_handle, &setup.institution_did, attrs).await;

        let (id, _delta, _timestamp) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, None, None).await.unwrap();
        assert_eq!(id, rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_rev_reg() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) = create_and_store_credential_def(setup.wallet_handle, &setup.institution_did, attrs).await;

        let (id, _rev_reg, _timestamp) = get_rev_reg(&rev_reg_id, time::get_time().sec as u64).await.unwrap();
        assert_eq!(id, rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_cred_def() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, cred_def_id, cred_def_json, _) =
            create_and_store_nonrevocable_credential_def(setup.wallet_handle, &setup.institution_did, attrs).await;

        let (id, cred_def) = get_cred_def(setup.pool_handle, None, &cred_def_id).await.unwrap();
        assert_eq!(id, cred_def_id);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&cred_def).unwrap(),
            serde_json::from_str::<serde_json::Value>(&cred_def_json).unwrap()
        );
    }

    #[tokio::test]
    async fn test_is_cred_def_on_ledger() {
        let setup = SetupWalletPool::init().await;

        assert_eq!(
            is_cred_def_on_ledger(setup.pool_handle, None, "V4SGRU86Z58d6TV7PBUe6f:3:CL:194:tag7")
                .await
                .unwrap(),
            false
        );
    }

    #[tokio::test]
    async fn from_pool_ledger_with_id() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _schema_json) =
            create_and_write_test_schema(setup.wallet_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;

        let rc = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await;

        let (_id, retrieved_schema) = rc.unwrap();
        assert!(retrieved_schema.contains(&schema_id));
    }

    #[tokio::test]
    async fn test_revoke_credential() {
        let setup = SetupWalletPool::init().await;

        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id) =
            create_and_store_credential(setup.wallet_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;

        let (_, first_rev_reg_delta, first_timestamp) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, None, None).await.unwrap();
        let (_, test_same_delta, test_same_timestamp) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, None, None).await.unwrap();

        assert_eq!(first_rev_reg_delta, test_same_delta);
        assert_eq!(first_timestamp, test_same_timestamp);

        revoke_credential(
            setup.wallet_handle,
            &setup.institution_did,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            &rev_reg_id,
            &cred_rev_id,
        )
        .await
        .unwrap();

        // Delta should change after revocation
        let (_, second_rev_reg_delta, _) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, Some(first_timestamp + 1), None)
            .await
            .unwrap();

        assert_ne!(first_rev_reg_delta, second_rev_reg_delta);
    }

    #[tokio::test]
    async fn test_get_txn() {
        let setup = SetupWalletPool::init().await;
        get_ledger_txn(setup.wallet_handle, None, 0).await.unwrap_err();
        let txn = get_ledger_txn(setup.wallet_handle, None, 1).await;
        assert!(txn.is_ok());

        get_ledger_txn(setup.wallet_handle, Some(&setup.institution_did), 0).await.unwrap_err();
        let txn = get_ledger_txn(setup.wallet_handle, Some(&setup.institution_did), 1).await;
        assert!(txn.is_ok());
    }
}
