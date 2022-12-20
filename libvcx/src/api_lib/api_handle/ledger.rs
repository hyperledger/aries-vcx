use std::sync::Arc;

use aries_vcx::common::ledger::transactions::{get_service, write_endpoint_legacy};
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::messages::did_doc::service_aries::AriesService;
use aries_vcx::messages::protocols::connection::did::Did;

use crate::api_lib::global::profile::get_main_profile;
use crate::api_lib::utils::libvcx_error::LibvcxResult;
use crate::api_lib::utils::mapping_ariesvcx_libvcx::map_ariesvcx_result;

pub async fn endorse_transaction(issuer_did: &str, transaction: &str) -> LibvcxResult<()> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.endorse_transaction(&issuer_did, &transaction).await)
}

pub async fn get_ledger_txn(seq_no: i32, submitter_did: Option<String>) -> LibvcxResult<String> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.get_ledger_txn(seq_no, submitter_did.as_deref()).await)
}

pub async fn rotate_verkey(did: &str) -> LibvcxResult<()> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    map_ariesvcx_result(aries_vcx::common::keys::rotate_verkey(&profile, &did).await)
}

pub async fn get_verkey_from_ledger(did: &str) -> LibvcxResult<String> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    map_ariesvcx_result(aries_vcx::common::keys::get_verkey_from_ledger(&profile, &did).await)
}

pub async fn ledger_write_endpoint_legacy(
    target_did: &str,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    endpoint: String,
)
    -> LibvcxResult<AriesService> {
    let service = AriesService::create()
        .set_service_endpoint(endpoint)
        .set_recipient_keys(recipient_keys)
        .set_routing_keys(routing_keys);
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    write_endpoint_legacy(&profile, target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_get_service(did: &Did) -> LibvcxResult<AriesService> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    map_ariesvcx_result(get_service(&profile, &did).await)
}

pub async fn ledger_get_txn_author_agreement() -> LibvcxResult<String> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
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
    map_ariesvcx_result(
        aries_vcx::utils::author_agreement::set_txn_author_agreement(
            text, version, hash, acc_mech_type, time_of_acceptance,
        )
    )
}
