use std::sync::Arc;

use aries_vcx::common::ledger::transactions::{get_service, write_endpoint_legacy};
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::messages::diddoc::aries::service::AriesService;
use aries_vcx::messages::protocols::connection::did::Did;

use crate::api_lib::errors::error::LibvcxResult;
use crate::api_lib::errors::mapping_from_ariesvcx::map_ariesvcx_result;
use crate::api_lib::global::profile::get_main_profile;

pub async fn endorse_transaction(issuer_did: &str, transaction: &str) -> LibvcxResult<()> {
    let profile = get_main_profile()?;
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.endorse_transaction(&issuer_did, &transaction).await)
}

pub async fn get_ledger_txn(seq_no: i32, submitter_did: Option<String>) -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.get_ledger_txn(seq_no, submitter_did.as_deref()).await)
}

pub async fn rotate_verkey(did: &str) -> LibvcxResult<()> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(aries_vcx::common::keys::rotate_verkey(&profile, &did).await)
}

pub async fn get_verkey_from_ledger(did: &str) -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(aries_vcx::common::keys::get_verkey_from_ledger(&profile, &did).await)
}

pub async fn ledger_write_endpoint_legacy(
    target_did: &str,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<AriesService> {
    let service = AriesService::create()
        .set_service_endpoint(endpoint)
        .set_recipient_keys(recipient_keys)
        .set_routing_keys(routing_keys);
    let profile = get_main_profile()?;
    write_endpoint_legacy(&profile, target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_get_service(did: &Did) -> LibvcxResult<AriesService> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(get_service(&profile, &did).await)
}

pub async fn ledger_get_txn_author_agreement() -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.get_txn_author_agreement().await)
}

pub fn ledger_set_txn_author_agreement(
    text: Option<String>,
    version: Option<String>,
    hash: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u64,
) -> LibvcxResult<()> {
    map_ariesvcx_result(aries_vcx::utils::author_agreement::set_txn_author_agreement(
        text,
        version,
        hash,
        acc_mech_type,
        time_of_acceptance,
    ))
}
