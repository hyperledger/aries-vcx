use aries_vcx::common::signing::unpack_message_to_string;
use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx::indy;
use aries_vcx::indy::wallet::{
    close_search_wallet, fetch_next_records_wallet, import, open_search_wallet, IssuerConfig, RestoreWalletConfigs,
    WalletConfig,
};
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::vdrtools::{SearchHandle, WalletHandle, INVALID_WALLET_HANDLE};

use crate::api_vcx::api_global::profile::{get_main_profile, get_main_wallet, indy_handles_to_profile};
use crate::errors::error::LibvcxResult;
use crate::errors::mapping_from_ariesvcx::map_ariesvcx_result;

pub static mut WALLET_HANDLE: WalletHandle = INVALID_WALLET_HANDLE;

pub fn set_main_wallet_handle(handle: WalletHandle) -> WalletHandle {
    trace!("set_wallet_handle >>> handle: {:?}", handle);
    unsafe {
        WALLET_HANDLE = handle;
    }
    unsafe { WALLET_HANDLE }
}

pub fn get_main_wallet_handle() -> WalletHandle {
    unsafe { WALLET_HANDLE }
}

pub fn reset_main_wallet_handle() {
    set_main_wallet_handle(INVALID_WALLET_HANDLE);
}

pub async fn export_main_wallet(path: &str, backup_key: &str) -> LibvcxResult<()> {
    map_ariesvcx_result(indy::wallet::export_wallet(get_main_wallet_handle(), path, backup_key).await)
}

pub async fn open_as_main_wallet(wallet_config: &WalletConfig) -> LibvcxResult<WalletHandle> {
    let handle = indy::wallet::open_wallet(wallet_config).await?;
    set_main_wallet_handle(handle);
    Ok(handle)
}

pub async fn create_and_open_as_main_wallet(wallet_config: &WalletConfig) -> LibvcxResult<WalletHandle> {
    let handle = indy::wallet::create_and_open_wallet(wallet_config).await?;
    set_main_wallet_handle(handle);
    Ok(handle)
}

pub async fn close_main_wallet() -> LibvcxResult<()> {
    indy::wallet::close_wallet(get_main_wallet_handle()).await?;
    reset_main_wallet_handle();
    Ok(())
}

pub async fn create_main_wallet(config: &WalletConfig) -> LibvcxResult<()> {
    let wallet_handle = create_and_open_as_main_wallet(config).await?;
    trace!("Created wallet with handle {:?}", wallet_handle);

    let profile = indy_handles_to_profile(wallet_handle, -1);

    // If MS is already in wallet then just continue
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .ok();

    close_main_wallet().await?;
    Ok(())
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use aries_vcx::global::settings::{CONFIG_WALLET_BACKUP_KEY, DEFAULT_WALLET_KEY, WALLET_KDF_RAW};
    use aries_vcx::indy::wallet::WalletConfig;
    use aries_vcx::utils::devsetup::TempFile;

    use crate::api_vcx::api_global::profile::indy_wallet_handle_to_wallet;
    use crate::api_vcx::api_global::settings::get_config_value;
    use crate::api_vcx::api_global::wallet::{close_main_wallet, create_and_open_as_main_wallet, export_main_wallet};

    fn _record() -> (&'static str, &'static str, &'static str) {
        ("type1", "id1", "value1")
    }

    pub async fn _create_main_wallet_and_its_backup() -> (TempFile, String, WalletConfig) {
        let wallet_name = &format!("export_test_wallet_{}", uuid::Uuid::new_v4());

        let export_file = TempFile::prepare_path(wallet_name);

        let wallet_config = WalletConfig {
            wallet_name: wallet_name.into(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let wallet_handle = create_and_open_as_main_wallet(&wallet_config).await.unwrap();
        let wallet = indy_wallet_handle_to_wallet(wallet_handle);
        wallet.create_and_store_my_did(None, None).await.unwrap();
        let backup_key = get_config_value(CONFIG_WALLET_BACKUP_KEY).unwrap();
        let (type_, id, value) = _record();
        wallet.add_wallet_record(type_, id, value, None).await.unwrap();
        export_main_wallet(&export_file.path, &backup_key).await.unwrap();

        close_main_wallet().await.unwrap();

        // todo: import and verify
        (export_file, wallet_name.to_string(), wallet_config)
    }
}

pub async fn key_for_local_did(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.key_for_local_did(did).await)
}

pub async fn wallet_sign(vk: &str, data_raw: &[u8]) -> LibvcxResult<Vec<u8>> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.sign(vk, data_raw).await)
}

pub async fn wallet_verify(vk: &str, msg: &[u8], signature: &[u8]) -> LibvcxResult<bool> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.verify(vk, msg, signature).await)
}

pub async fn replace_did_keys_start(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.replace_did_keys_start(did).await)
}

pub async fn rotate_verkey_apply(did: &str, temp_vk: &str) -> LibvcxResult<()> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(aries_vcx::common::keys::rotate_verkey_apply(&profile, did, temp_vk).await)
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
    map_ariesvcx_result(indy::wallet::wallet_configure_issuer(wallet, enterprise_seed).await)
}

pub async fn wallet_add_wallet_record(type_: &str, id: &str, value: &str, option: Option<&str>) -> LibvcxResult<()> {
    let wallet = get_main_wallet();
    map_ariesvcx_result(wallet.add_wallet_record(type_, id, value, option).await)
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

pub async fn wallet_open_search_wallet(
    xtype: &str,
    query_json: &str,
    options_json: &str,
) -> LibvcxResult<SearchHandle> {
    // TODO - future - use profile wallet to stop binding to indy
    let wallet_handle = get_main_wallet_handle();
    map_ariesvcx_result(open_search_wallet(wallet_handle, xtype, query_json, options_json).await)
}

pub async fn wallet_close_search_wallet(wallet_search_handle: SearchHandle) -> LibvcxResult<()> {
    map_ariesvcx_result(close_search_wallet(wallet_search_handle).await)
}

pub async fn wallet_fetch_next_records_wallet(
    wallet_search_handle: SearchHandle,
    count: usize,
) -> LibvcxResult<String> {
    // TODO - future - use profile wallet to stop binding to indy
    let wallet_handle = get_main_wallet_handle();
    map_ariesvcx_result(fetch_next_records_wallet(wallet_handle, wallet_search_handle, count).await)
}

pub async fn wallet_import(config: &RestoreWalletConfigs) -> LibvcxResult<()> {
    map_ariesvcx_result(import(config).await)
}
