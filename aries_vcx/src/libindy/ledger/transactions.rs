use std::collections::HashMap;

use futures::future::TryFutureExt;
use vdrtools::cache;
use vdrtools::ledger;
use vdrtools_sys::{PoolHandle, WalletHandle};
use serde_json;

use messages::did_doc::service_aries::AriesService;
use crate::error::prelude::*;
use crate::global::settings;
use crate::libindy::utils::mocks::pool_mocks::PoolMocks;
use crate::libindy::keys::create_and_store_my_did;
use messages::connection::did::Did;
use messages::connection::invite::Invitation;
use crate::utils;
use crate::utils::constants::{CRED_DEF_ID, CRED_DEF_JSON, CRED_DEF_REQ, rev_def_json, REV_REG_DELTA_JSON, REV_REG_ID, REV_REG_JSON, REVOC_REG_TYPE, SCHEMA_ID, SCHEMA_JSON, SCHEMA_TXN, SUBMIT_SCHEMA_RESPONSE};
use crate::utils::random::generate_random_did;
use messages::did_doc::service_resolvable::ServiceResolvable;
use messages::did_doc::DidDoc;

pub async fn multisign_request(wallet_handle: WalletHandle, did: &str, request: &str) -> VcxResult<String> {
    ledger::multi_sign_request(wallet_handle, did, request)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_sign_request(wallet_handle: WalletHandle, did: &str, request: &str) -> VcxResult<String> {
    ledger::sign_request(wallet_handle, did, request)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_sign_and_submit_request(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    request_json: &str,
) -> VcxResult<String> {
    trace!(
        "libindy_sign_and_submit_request >>> issuer_did: {}, request_json: {}",
        issuer_did,
        request_json
    );
    if settings::indy_mocks_enabled() {
        return Ok(r#"{"rc":"success"}"#.to_string());
    }
    if PoolMocks::has_pool_mock_responses() {
        warn!("libindy_sign_and_submit_request >> retrieving pool mock response");
        return Ok(PoolMocks::get_next_pool_response());
    };

    ledger::sign_and_submit_request(pool_handle, wallet_handle, issuer_did, request_json)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_submit_request(pool_handle: PoolHandle, request_json: &str) -> VcxResult<String> {
    trace!("libindy_submit_request >>> request_json: {}", request_json);
    ledger::submit_request(pool_handle, request_json)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_schema_request(submitter_did: &str, data: &str) -> VcxResult<String> {
    trace!(
        "libindy_build_schema_request >>> submitter_did: {}, data: {}",
        submitter_did,
        data
    );
    ledger::build_schema_request(submitter_did, data)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_create_credential_def_txn(
    submitter_did: &str,
    credential_def_json: &str,
) -> VcxResult<String> {
    trace!(
        "libindy_build_create_credential_def_txn >>> submitter_did: {}, credential_def_json: {}",
        submitter_did,
        credential_def_json
    );
    ledger::build_cred_def_request(submitter_did, credential_def_json)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_get_txn_author_agreement(pool_handle: PoolHandle) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string());
    }

    let did = generate_random_did();

    let get_author_agreement_request = ledger::build_get_txn_author_agreement_request(Some(&did), None).await?;

    let get_author_agreement_response = libindy_submit_request(pool_handle, &get_author_agreement_request).await?;

    let get_author_agreement_response = serde_json::from_str::<serde_json::Value>(&get_author_agreement_response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

    let mut author_agreement_data = get_author_agreement_response["result"]["data"]
        .as_object()
        .map_or(json!({}), |data| json!(data));

    let get_acceptance_mechanism_request =
        ledger::build_get_acceptance_mechanisms_request(Some(&did), None, None).await?;

    let get_acceptance_mechanism_response = libindy_submit_request(pool_handle, &get_acceptance_mechanism_request).await?;

    let get_acceptance_mechanism_response =
        serde_json::from_str::<serde_json::Value>(&get_acceptance_mechanism_response)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;

    if let Some(aml) = get_acceptance_mechanism_response["result"]["data"]["aml"].as_object() {
        author_agreement_data["aml"] = json!(aml);
    }

    Ok(author_agreement_data.to_string())
}

pub async fn append_txn_author_agreement_to_request(request_json: &str) -> VcxResult<String> {
    trace!("append_txn_author_agreement_to_request >>> request_json: ...");
    if let Some(author_agreement) = utils::author_agreement::get_txn_author_agreement()? {
        ledger::append_txn_author_agreement_acceptance_to_request(
            request_json,
            author_agreement.text.as_deref(),
            author_agreement.version.as_deref(),
            author_agreement.taa_digest.as_deref(),
            &author_agreement.acceptance_mechanism_type,
            author_agreement.time_of_acceptance,
        )
        .map_err(VcxError::from)
        .await
    } else {
        Ok(request_json.to_string())
    }
}

pub async fn libindy_build_auth_rules_request(submitter_did: &str, data: &str) -> VcxResult<String> {
    ledger::build_auth_rules_request(submitter_did, data)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_attrib_request(
    submitter_did: &str,
    target_did: &str,
    hash: Option<&str>,
    raw: Option<&str>,
    enc: Option<&str>,
) -> VcxResult<String> {
    ledger::build_attrib_request(submitter_did, target_did, hash, raw, enc)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_get_auth_rule_request(
    submitter_did: Option<&str>,
    txn_type: Option<&str>,
    action: Option<&str>,
    field: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
) -> VcxResult<String> {
    ledger::build_get_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_get_nym_request(submitter_did: Option<&str>, did: &str) -> VcxResult<String> {
    ledger::build_get_nym_request(submitter_did, did)
        .map_err(VcxError::from)
        .await
}

pub async fn libindy_build_nym_request(
    submitter_did: &str,
    target_did: &str,
    verkey: Option<&str>,
    data: Option<&str>,
    role: Option<&str>,
) -> VcxResult<String> {
    if PoolMocks::has_pool_mock_responses() {
        warn!("libindy_build_nym_request >> retrieving pool mock response");
        Ok(PoolMocks::get_next_pool_response())
    } else {
        ledger::build_nym_request(submitter_did, target_did, verkey, data, role)
            .map_err(VcxError::from)
            .await
    }
}

pub async fn get_nym(pool_handle: PoolHandle, did: &str) -> VcxResult<String> {
    let submitter_did = generate_random_did();

    let get_nym_req = libindy_build_get_nym_request(Some(&submitter_did), did).await?;
    libindy_submit_request(pool_handle, &get_nym_req).await
}

pub fn parse_response(response: &str) -> VcxResult<Response> {
    serde_json::from_str::<Response>(response)
        .to_vcx(VcxErrorKind::InvalidJson, "Cannot deserialize transaction response")
}

pub async fn libindy_get_schema(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    submitter_did: &str,
    schema_id: &str,
) -> VcxResult<String> {
    cache::get_schema(pool_handle, wallet_handle, submitter_did, schema_id, "{}")
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_build_get_cred_def_request(submitter_did: Option<&str>, cred_def_id: &str) -> VcxResult<String> {
    ledger::build_get_cred_def_request(submitter_did, cred_def_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_get_cred_def(wallet_handle: WalletHandle, pool_handle: PoolHandle, cred_def_id: &str) -> VcxResult<String> {
    let submitter_did = generate_random_did();
    trace!(
        "libindy_get_cred_def >>> pool_handle: {}, wallet_handle: {:?}, submitter_did: {}",
        pool_handle,
        wallet_handle,
        submitter_did
    );

    cache::get_cred_def(pool_handle, wallet_handle, &submitter_did, cred_def_id, "{}")
        .await
        .map_err(VcxError::from)
}

pub async fn set_endorser(wallet_handle: WalletHandle, submitter_did: &str, request: &str, endorser: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(utils::constants::REQUEST_WITH_ENDORSER.to_string());
    }

    let request = ledger::append_request_endorser(request, endorser).await?;

    multisign_request(wallet_handle, &submitter_did, &request).await
}

pub async fn endorse_transaction(wallet_handle: WalletHandle, pool_handle: PoolHandle, endorser_did: &str, transaction_json: &str) -> VcxResult<()> {
    //TODO Potentially VCX should handle case when endorser would like to pay fee
    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    _verify_transaction_can_be_endorsed(transaction_json, &endorser_did)?;

    let transaction = multisign_request(wallet_handle, &endorser_did, transaction_json).await?;
    let response = libindy_submit_request(pool_handle, &transaction).await?;

    match parse_response(&response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(VcxError::from_msg(
            VcxErrorKind::PostMessageFailed,
            format!("{:?}", res.reason),
        )),
    }
}

fn _verify_transaction_can_be_endorsed(transaction_json: &str, _did: &str) -> VcxResult<()> {
    let transaction: Request = serde_json::from_str(transaction_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("{:?}", err)))?;

    let transaction_endorser = transaction.endorser.ok_or(VcxError::from_msg(
        VcxErrorKind::InvalidJson,
        "Transaction cannot be endorsed: endorser DID is not set.",
    ))?;

    if transaction_endorser != _did {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!(
                "Transaction cannot be endorsed: transaction endorser DID `{}` and sender DID `{}` are different",
                transaction_endorser, _did
            ),
        ));
    }

    let identifier = transaction.identifier.as_str();
    if transaction.signature.is_none()
        && !transaction
            .signatures
            .as_ref()
            .map(|signatures| signatures.contains_key(identifier))
            .unwrap_or(false)
    {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "Transaction cannot be endorsed: the author must sign the transaction.".to_string(),
        ));
    }

    Ok(())
}

pub async fn build_attrib_request(
    submitter_did: &str,
    target_did: &str,
    hash: Option<&str>,
    raw: Option<&str>,
    enc: Option<&str>,
) -> VcxResult<String> {
    trace!(
        "build_attrib_request >>> submitter_did: {}, target_did: {}, hash: {:?}, raw: {:?}, enc: {:?}",
        submitter_did,
        target_did,
        hash,
        raw,
        enc
    );
    if settings::indy_mocks_enabled() {
        return Ok("{}".into());
    }
    let request = libindy_build_attrib_request(submitter_did, target_did, hash, raw, enc).await?;
    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
}

pub async fn add_attr(wallet_handle: WalletHandle, pool_handle: PoolHandle, did: &str, attrib_json: &str) -> VcxResult<String> {
    trace!("add_attr >>> did: {}, attrib_json: {}", did, attrib_json);
    let attrib_req = build_attrib_request(did, did, None, Some(attrib_json), None).await?;
    libindy_sign_and_submit_request(wallet_handle, pool_handle, did, &attrib_req).await
}

pub async fn get_attr(pool_handle: PoolHandle, did: &str, attr_name: &str) -> VcxResult<String> {
    let get_attrib_req = ledger::build_get_attrib_request(None, did, Some(attr_name), None, None).await?;
    libindy_submit_request(pool_handle, &get_attrib_req).await
}

pub async fn get_service(pool_handle: PoolHandle, did: &Did) -> VcxResult<AriesService> {
    let attr_resp = get_attr(pool_handle, &did.to_string(), "service").await?;
    let data = get_data_from_response(&attr_resp)?;
    let ser_service = match data["service"].as_str() {
        Some(ser_service) => ser_service.to_string(),
        None => {
            warn!("Failed converting service read from ledger {:?} to string, falling back to new single-serialized format", data["service"]);
            data["service"].to_string()
        }
    };
    serde_json::from_str(&ser_service).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to deserialize service read from the ledger: {:?}", err),
        )
    })
}

pub async fn resolve_service(pool_handle: PoolHandle, service: &ServiceResolvable) -> VcxResult<AriesService> {
    match service {
        ServiceResolvable::AriesService(service) => Ok(service.clone()),
        ServiceResolvable::Did(did) => get_service(pool_handle, did).await,
    }
}


pub async fn into_did_doc(pool_handle: PoolHandle, invitation: &Invitation) -> VcxResult<DidDoc> {
    let mut did_doc: DidDoc = DidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = match invitation {
        Invitation::Public(invitation) => {
            did_doc.set_id(invitation.did.to_string());
            let service = get_service(pool_handle, &invitation.did).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
        Invitation::Pairwise(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            (
                invitation.service_endpoint.clone(),
                invitation.recipient_keys.clone(),
                invitation.routing_keys.clone(),
            )
        }
        Invitation::OutOfBand(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            let service = resolve_service(pool_handle, &invitation.services[0]).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}

pub async fn add_service(wallet_handle: WalletHandle, pool_handle: PoolHandle, did: &str, service: &AriesService) -> VcxResult<String> {
    let attrib_json = json!({ "service": service }).to_string();
    add_attr(wallet_handle, pool_handle, did, &attrib_json).await
}

fn get_data_from_response(resp: &str) -> VcxResult<serde_json::Value> {
    let resp: serde_json::Value = serde_json::from_str(resp)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;
    serde_json::from_str(resp["result"]["data"].as_str().unwrap_or("{}"))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub req_id: u64,
    pub identifier: String,
    pub signature: Option<String>,
    pub signatures: Option<HashMap<String, String>>,
    pub endorser: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum Response {
    #[serde(rename = "REQNACK")]
    ReqNACK(Reject),
    #[serde(rename = "REJECT")]
    Reject(Reject),
    #[serde(rename = "REPLY")]
    Reply(Reply),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reject {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    ReplyV0(ReplyV0),
    ReplyV1(ReplyV1),
}

#[derive(Debug, Deserialize)]
pub struct ReplyV0 {
    pub result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1 {
    pub data: ReplyDataV1,
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1 {
    pub result: serde_json::Value,
}

pub async fn sign_and_submit_to_ledger(wallet_handle: WalletHandle, pool_handle: PoolHandle, submitter_did: &str, req: &str) -> VcxResult<String> {
    debug!("sign_and_submit_to_ledger(submitter_did: {}, req: {}", submitter_did, req);
    if settings::indy_mocks_enabled() {
        return Ok(SUBMIT_SCHEMA_RESPONSE.to_string());
    }
    let response = libindy_sign_and_submit_request(wallet_handle, pool_handle, &submitter_did, req).await?;
    debug!("sign_and_submit_to_ledger >>> response: {}", &response);
    Ok(response)
}

pub async fn add_new_did(wallet_handle: WalletHandle, pool_handle: PoolHandle, submitter_did: &str, role: Option<&str>) -> (String, String) {
    let (did, verkey) = create_and_store_my_did(wallet_handle, None, None).await.unwrap();
    let mut req_nym = ledger::build_nym_request(&submitter_did, &did, Some(&verkey), None, role)
        .await
        .unwrap();

    req_nym = append_txn_author_agreement_to_request(&req_nym).await.unwrap();

    libindy_sign_and_submit_request(wallet_handle, pool_handle, &submitter_did, &req_nym)
        .await
        .unwrap();
    (did, verkey)
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

pub async fn build_schema_request(submitter_did: &str, schema: &str) -> VcxResult<String> {
    trace!("build_schema_request >>> submitter_did: {}, schema: {}", submitter_did, schema);

    if settings::indy_mocks_enabled() {
        return Ok(SCHEMA_TXN.to_string());
    }

    let request = libindy_build_schema_request(&submitter_did, schema).await?;

    let request = append_txn_author_agreement_to_request(&request).await?;

    Ok(request)
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

pub async fn get_rev_reg_def_json(pool_handle: PoolHandle, rev_reg_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        debug!("get_rev_reg_def_json >>> returning mocked value");
        return Ok((REV_REG_ID.to_string(), rev_def_json()));
    }

    let submitter_did = generate_random_did();

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

    let submitter_did = generate_random_did();

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

pub async fn get_rev_reg(pool_handle: PoolHandle, rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
    if settings::indy_mocks_enabled() {
        return Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1));
    }

    let submitter_did = generate_random_did();

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
    pool_handle: PoolHandle,
    submitter_did: Option<&str>,
    seq_no: i32,
) -> VcxResult<String> {
    trace!(
        "get_ledger_txn >>> submitter_did: {:?}, seq_no: {}",
        submitter_did,
        seq_no
    );
    let req = build_get_txn_request(submitter_did, seq_no).await?;
    let res = if let Some(submitter_did) = submitter_did {
        libindy_sign_and_submit_request(wallet_handle, pool_handle, submitter_did, &req).await?
    } else {
        libindy_submit_request(pool_handle, &req).await?
    };
    check_response(&res)?;
    Ok(res)
}

pub fn _check_schema_response(response: &str) -> VcxResult<()> {
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

pub(in crate::libindy) fn check_response(response: &str) -> VcxResult<()> {
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!("{:?}", res),
        )),
    }
}

pub async fn get_schema_json(wallet_handle: WalletHandle, pool_handle: PoolHandle, schema_id: &str) -> VcxResult<(String, String)> {
    trace!("get_schema_json >>> schema_id: {}", schema_id);
    if settings::indy_mocks_enabled() {
        return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
    }

    let submitter_did = generate_random_did();

    let schema_json = libindy_get_schema(wallet_handle, pool_handle, &submitter_did, schema_id).await?;

    Ok((schema_id.to_string(), schema_json))
}

pub async fn build_cred_def_request(issuer_did: &str, cred_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(CRED_DEF_REQ.to_string());
    }

    let cred_def_req = libindy_build_create_credential_def_txn(issuer_did, cred_def_json).await?;

    let cred_def_req = append_txn_author_agreement_to_request(&cred_def_req).await?;

    Ok(cred_def_req)
}

pub async fn get_cred_def_json(wallet_handle: WalletHandle, pool_handle: PoolHandle, cred_def_id: &str) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        debug!("get_cred_def_json >>> returning mocked value");
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let cred_def_json = libindy_get_cred_def(wallet_handle, pool_handle, cred_def_id).await?;

    Ok((cred_def_id.to_string(), cred_def_json))
}


#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use crate::utils::devsetup::*;
    use messages::a2a::MessageId;
    use messages::did_doc::test_utils::{_recipient_keys, _routing_keys, _service_endpoint};
    use messages::connection::invite::test_utils::_pairwise_invitation;

    use super::*;

    #[test]
    fn test_verify_transaction_can_be_endorsed() {
        let _setup = SetupDefaults::init();

        // success
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "signature": "gkVDhwe2", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_ok());

        // no author signature
        let transaction =
            r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_err());

        // different endorser did
        let transaction =
            r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "EbP4aYNeTHL6q385GuVpRV").is_err());
    }

    #[tokio::test]
    async fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());
        assert_eq!(did_doc, into_did_doc(0, &Invitation::Pairwise(_pairwise_invitation())).await.unwrap());
    }
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::libindy::ledger::transactions::get_ledger_txn;
    use crate::utils::devsetup::SetupWalletPool;

    #[tokio::test]
    async fn test_get_txn() {
        let setup = SetupWalletPool::init().await;
        get_ledger_txn(setup.wallet_handle, setup.pool_handle, None, 0).await.unwrap_err();
        let txn = get_ledger_txn(setup.wallet_handle, setup.pool_handle, None, 1).await;
        assert!(txn.is_ok());

        get_ledger_txn(setup.wallet_handle, setup.pool_handle, Some(&setup.institution_did), 0).await.unwrap_err();
        let txn = get_ledger_txn(setup.wallet_handle, setup.pool_handle, Some(&setup.institution_did), 1).await;
        assert!(txn.is_ok());
    }
}

