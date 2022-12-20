use aries_vcx::common::signing::unpack_message_to_string;
use aries_vcx::errors::error::VcxResult;
use aries_vcx::indy;
use aries_vcx::indy::wallet::{close_search_wallet, fetch_next_records_wallet, import, IssuerConfig, open_search_wallet, RestoreWalletConfigs};
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::vdrtools::SearchHandle;

use crate::api_lib::global::profile::{get_main_profile, get_main_wallet};
use crate::api_lib::global::wallet::get_main_wallet_handle;
use crate::api_lib::errors::error_libvcx::LibvcxResult;
use crate::api_lib::errors::mapping_from_ariesvcx::map_ariesvcx_result;

pub async fn key_for_local_did(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.key_for_local_did(did).await)
}

pub async fn wallet_sign(vk: &str, data_raw: &[u8]) -> LibvcxResult<Vec<u8>> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.sign(&vk, &data_raw).await)
}

pub async fn wallet_verify(vk: &str, msg: &[u8], signature: &[u8]) -> LibvcxResult<bool> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.verify(vk, msg, signature).await)
}

pub async fn replace_did_keys_start(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.replace_did_keys_start(&did).await)
}

pub async fn rotate_verkey_apply(did: &str, temp_vk: &str) -> LibvcxResult<()> {
    let profile = match get_main_profile() {
        Ok(profile) => profile,
        Err(err) => return Err(err)
    };
    map_ariesvcx_result(aries_vcx::common::keys::rotate_verkey_apply(&profile, &did, &temp_vk).await)
}

pub async fn wallet_unpack_message_to_string(payload: &[u8]) -> LibvcxResult<String> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(unpack_message_to_string(&wallet, payload).await)
}

pub async fn wallet_create_pairwise_did() -> LibvcxResult<PairwiseInfo> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(PairwiseInfo::create(&wallet).await)
}

pub async fn wallet_configure_issuer(enterprise_seed: &str) -> LibvcxResult<IssuerConfig> {
    // TODO - future - use profile wallet to stop indy dependency
    let wallet = get_main_wallet_handle();
    map_ariesvcx_result(indy::wallet::wallet_configure_issuer(wallet, &enterprise_seed).await)
}

pub async fn wallet_add_wallet_record(type_: &str, id: &str, value: &str, option: Option<&str>) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.add_wallet_record(&type_, &id, &value, option).await)
}

pub async fn wallet_update_wallet_record_value(xtype: &str, id: &str, value: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.update_wallet_record_value(xtype, id, value).await)
}

pub async fn wallet_update_wallet_record_tags(xtype: &str, id: &str, tags_json: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.update_wallet_record_tags(xtype, id, tags_json).await)
}

pub async fn wallet_add_wallet_record_tags(xtype: &str, id: &str, tags_json: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.add_wallet_record_tags(xtype, id, tags_json).await)
}

pub async fn wallet_delete_wallet_record_tags(xtype: &str, id: &str, tags_json: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.delete_wallet_record_tags(xtype, id, tags_json).await)
}

pub async fn wallet_get_wallet_record(xtype: &str, id: &str, options: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.get_wallet_record(xtype, id, options).await)
}

pub async fn wallet_delete_wallet_record(xtype: &str, id: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.delete_wallet_record(xtype, id).await)
}

pub async fn wallet_open_search_wallet(xtype: &str, query_json: &str, options_json: &str) -> LibvcxResult<SearchHandle> {
    // TODO - future - use profile wallet to stop binding to indy
    let wallet_handle = get_main_wallet_handle();
    map_ariesvcx_result(open_search_wallet(wallet_handle, &xtype, &query_json, &options_json).await)
}

pub async fn wallet_close_search_wallet(wallet_search_handle: SearchHandle) -> LibvcxResult<()> {
    map_ariesvcx_result(close_search_wallet(wallet_search_handle).await)
}

pub async fn wallet_fetch_next_records_wallet(wallet_search_handle: SearchHandle, count: usize) -> LibvcxResult<String> {
    // TODO - future - use profile wallet to stop binding to indy
    let wallet_handle = get_main_wallet_handle();
    map_ariesvcx_result(fetch_next_records_wallet(wallet_handle, wallet_search_handle, count).await)
}

pub async fn wallet_import(config: &RestoreWalletConfigs) -> LibvcxResult<()> {
    map_ariesvcx_result(import(&config).await)
}





