use napi_derive::napi;

use crate::error::to_napi_err;
use vcx::api_vcx::api_global::ledger;
use vcx::serde_json::json;

#[napi]
async fn get_ledger_author_agreement() -> napi::Result<String> {
    let res = ledger::ledger_get_txn_author_agreement().await.map_err(to_napi_err)?;
    Ok(res)
}

// todo: ideally time_of_acceptance is u64, but napi doesn't support u64
#[napi]
fn set_active_txn_author_agreement_meta(
    text: Option<String>,
    version: Option<String>,
    hash: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u32,
) -> napi::Result<()> {
    ledger::ledger_set_txn_author_agreement(text, version, hash, acc_mech_type, time_of_acceptance as u64)
        .map_err(to_napi_err)
}

#[napi]
async fn create_service(
    target_did: String,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    endpoint: String,
) -> napi::Result<String> {
    let res = ledger::ledger_write_endpoint_legacy(&target_did, recipient_keys, routing_keys, endpoint)
        .await
        .map_err(to_napi_err)?;
    Ok(json!(res).to_string())
}

#[napi]
async fn get_service_from_ledger(target_did: String) -> napi::Result<String> {
    let res = ledger::ledger_get_service(&target_did).await.map_err(to_napi_err)?;
    Ok(json!(res).to_string())
}

#[napi]
async fn get_verkey_from_ledger(did: String) -> napi::Result<String> {
    ledger::get_verkey_from_ledger(&did).await.map_err(to_napi_err)
}

#[napi]
async fn get_ledger_txn(seq_no: i32, submitter_did: Option<String>) -> napi::Result<String> {
    ledger::get_ledger_txn(seq_no, submitter_did).await.map_err(to_napi_err)
}
