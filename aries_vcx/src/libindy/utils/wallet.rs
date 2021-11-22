use indy::{ErrorCode, wallet};
use indy::{INVALID_WALLET_HANDLE, SearchHandle, WalletHandle};
use indy::future::Future;
use indy_facade::wallet::{build_wallet_config, build_wallet_credentials, create_indy_wallet, WalletConfig, WalletCredentials};

use crate::error::prelude::*;
use crate::init::open_as_main_wallet;
use crate::libindy::utils::{anoncreds, signus};
use crate::settings;


#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct IssuerConfig {
    pub institution_did: String,
    pub institution_verkey: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletRecord {
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    pub value: Option<String>,
    tags: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreWalletConfigs {
    pub wallet_name: String,
    pub wallet_key: String,
    pub exported_wallet_path: String,
    pub backup_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_key_derivation: Option<String>,
}

impl RestoreWalletConfigs {
    pub fn from_str(data: &str) -> VcxResult<RestoreWalletConfigs> {
        serde_json::from_str(data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize RestoreWalletConfigs: {:?}", err)))
    }
}

pub static mut WALLET_HANDLE: WalletHandle = INVALID_WALLET_HANDLE;

pub fn set_wallet_handle(handle: WalletHandle) -> WalletHandle {
    trace!("set_wallet_handle >>> handle: {:?}", handle);
    unsafe { WALLET_HANDLE = handle; }
    settings::get_agency_client_mut().unwrap().set_wallet_handle(handle.0);
    unsafe { WALLET_HANDLE }
}

pub fn get_wallet_handle() -> WalletHandle { unsafe { WALLET_HANDLE } }

pub fn reset_wallet_handle() -> VcxResult<()> {
    set_wallet_handle(INVALID_WALLET_HANDLE);
    settings::get_agency_client_mut()?.reset_wallet_handle();
    Ok(())
}

pub fn create_wallet(config: &WalletConfig) -> VcxResult<()> {
    let wh = create_and_open_as_main_wallet(&config)?;
    trace!("Created wallet with handle {:?}", wh);

    // If MS is already in wallet then just continue
    anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).ok();

    close_main_wallet()?;
    Ok(())
}

pub fn configure_issuer_wallet(enterprise_seed: &str) -> VcxResult<IssuerConfig> {
    let (institution_did, institution_verkey) = signus::create_and_store_my_did(Some(enterprise_seed), None)?;
    Ok(IssuerConfig {
        institution_did,
        institution_verkey,
    })
}

pub fn create_and_open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    if settings::indy_mocks_enabled() {
        warn!("open_as_main_wallet ::: Indy mocks enabled, skipping opening main wallet.");
        return Ok(set_wallet_handle(WalletHandle(1)));
    }

    create_indy_wallet(&wallet_config)?;
    open_as_main_wallet(&wallet_config)
}

pub fn close_main_wallet() -> VcxResult<()> {
    trace!("close_main_wallet >>>");
    if settings::indy_mocks_enabled() {
        warn!("close_main_wallet >>> Indy mocks enabled, skipping closing wallet");
        set_wallet_handle(INVALID_WALLET_HANDLE);
        return Ok(());
    }

    wallet::close_wallet(get_wallet_handle())
        .wait()?;

    reset_wallet_handle()?;
    Ok(())
}

pub fn add_record(xtype: &str, id: &str, value: &str, tags: Option<&str>) -> VcxResult<()> {
    trace!("add_record >>> xtype: {}, id: {}, value: {}, tags: {:?}", secret!(&xtype), secret!(&id), secret!(&value), secret!(&tags));

    if settings::indy_mocks_enabled() { return Ok(()); }

    wallet::add_wallet_record(get_wallet_handle(), xtype, id, value, tags)
        .wait()
        .map_err(VcxError::from)
}

pub fn get_record(xtype: &str, id: &str, options: &str) -> VcxResult<String> {
    trace!("get_record >>> xtype: {}, id: {}, options: {}", secret!(&xtype), secret!(&id), options);

    if settings::indy_mocks_enabled() {
        return Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string());
    }

    wallet::get_wallet_record(get_wallet_handle(), xtype, id, options)
        .wait()
        .map_err(VcxError::from)
}

pub fn delete_record(xtype: &str, id: &str) -> VcxResult<()> {
    trace!("delete_record >>> xtype: {}, id: {}", secret!(&xtype), secret!(&id));

    if settings::indy_mocks_enabled() { return Ok(()); }

    wallet::delete_wallet_record(get_wallet_handle(), xtype, id)
        .wait()
        .map_err(VcxError::from)
}


pub fn update_record_value(xtype: &str, id: &str, value: &str) -> VcxResult<()> {
    trace!("update_record_value >>> xtype: {}, id: {}, value: {}", secret!(&xtype), secret!(&id), secret!(&value));

    if settings::indy_mocks_enabled() { return Ok(()); }

    wallet::update_wallet_record_value(get_wallet_handle(), xtype, id, value)
        .wait()
        .map_err(VcxError::from)
}

pub fn add_record_tags(xtype: &str, id: &str, tags: &str) -> VcxResult<()> {
    trace!("add_record_tags >>> xtype: {}, id: {}, tags: {:?}", secret!(&xtype), secret!(&id), secret!(&tags));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    wallet::add_wallet_record_tags(get_wallet_handle(), xtype, id, tags)
        .wait()
        .map_err(VcxError::from)
}

pub fn update_record_tags(xtype: &str, id: &str, tags: &str) -> VcxResult<()> {
    trace!("update_record_tags >>> xtype: {}, id: {}, tags: {}", secret!(&xtype), secret!(&id), secret!(&tags));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    wallet::update_wallet_record_tags(get_wallet_handle(), xtype, id, tags)
        .wait()
        .map_err(VcxError::from)
}

pub fn delete_record_tags(xtype: &str, id: &str, tag_names: &str) -> VcxResult<()> {
    trace!("delete_record_tags >>> xtype: {}, id: {}, tag_names: {}", secret!(&xtype), secret!(&id), secret!(&tag_names));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    wallet::delete_wallet_record_tags(get_wallet_handle(), xtype, id, tag_names)
        .wait()
        .map_err(VcxError::from)
}

pub fn open_search(xtype: &str, query: &str, options: &str) -> VcxResult<SearchHandle> {
    trace!("open_search >>> xtype: {}, query: {}, options: {}", secret!(&xtype), query, options);

    if settings::indy_mocks_enabled() {
        return Ok(1);
    }

    wallet::open_wallet_search(get_wallet_handle(), xtype, query, options)
        .wait()
        .map_err(VcxError::from)
}

pub fn fetch_next_records(search_handle: SearchHandle, count: usize) -> VcxResult<String> {
    trace!("fetch_next_records >>> search_handle: {}, count: {}", search_handle, count);

    if settings::indy_mocks_enabled() {
        return Ok(String::from("{}"));
    }

    wallet::fetch_wallet_search_next_records(get_wallet_handle(), search_handle, count)
        .wait()
        .map_err(VcxError::from)
}

pub fn close_search(search_handle: SearchHandle) -> VcxResult<()> {
    trace!("close_search >>> search_handle: {}", search_handle);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    wallet::close_wallet_search(search_handle)
        .wait()
        .map_err(VcxError::from)
}

pub fn export_main_wallet(path: &str, backup_key: &str) -> VcxResult<()> {
    let wallet_handle = get_wallet_handle();
    trace!("export >>> wallet_handle: {:?}, path: {:?}, backup_key: ****", wallet_handle, path);

    let export_config = json!({ "key": backup_key, "path": &path}).to_string();
    wallet::export_wallet(wallet_handle, &export_config)
        .wait()
        .map_err(VcxError::from)
}

pub fn import(restore_config: &RestoreWalletConfigs) -> VcxResult<()> {
    trace!("import >>> wallet: {} exported_wallet_path: {}", restore_config.wallet_name, restore_config.exported_wallet_path);
    let new_wallet_name = restore_config.wallet_name.clone();
    let new_wallet_key = restore_config.wallet_key.clone();
    let new_wallet_kdf = restore_config.wallet_key_derivation.clone().unwrap_or(settings::WALLET_KDF_DEFAULT.into());

    let new_wallet_config = build_wallet_config(&new_wallet_name, None, None);
    let new_wallet_credentials = build_wallet_credentials(&new_wallet_key, None, &new_wallet_kdf, None, None)?;
    let import_config = json!({
        "key": restore_config.backup_key,
        "path": restore_config.exported_wallet_path
    }).to_string();

    wallet::import_wallet(&new_wallet_config, &new_wallet_credentials, &import_config)
        .wait()
        .map_err(VcxError::from)
}

#[cfg(feature = "test_utils")]
pub mod tests {
    use crate::libindy::utils::signus::create_and_store_my_did;
    use crate::utils::devsetup::TempFile;

    use super::*;

    fn _record() -> (&'static str, &'static str, &'static str) {
        ("type1", "id1", "value1")
    }

    pub fn create_main_wallet_and_its_backup() -> (TempFile, String, WalletConfig) {
        let wallet_name = &format!("export_test_wallet_{}", uuid::Uuid::new_v4());

        let export_file = TempFile::prepare_path(wallet_name);

        let wallet_config = WalletConfig {
            wallet_name: wallet_name.into(),
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let _handle = create_and_open_as_main_wallet(&wallet_config).unwrap();

        let (my_did, my_vk) = create_and_store_my_did(None, None).unwrap();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
        settings::get_agency_client_mut().unwrap().set_my_vk(&my_vk);

        let backup_key = settings::get_config_value(settings::CONFIG_WALLET_BACKUP_KEY).unwrap();

        let (type_, id, value) = _record();
        add_record(type_, id, value, None).unwrap();

        export_main_wallet(&export_file.path, &backup_key).unwrap();

        close_main_wallet().unwrap();

        (export_file, wallet_name.to_string(), wallet_config)
    }
}
