use crate::{global, indy, libindy};
use crate::error::{VcxError, VcxResult};
use crate::global::settings;
use crate::indy::{INVALID_WALLET_HANDLE, SearchHandle, WalletHandle};
use crate::libindy::utils::{anoncreds, signus, wallet};
use crate::libindy::utils::wallet::{IssuerConfig, WalletConfig};

pub static mut WALLET_HANDLE: WalletHandle = INVALID_WALLET_HANDLE;

pub fn set_wallet_handle(handle: WalletHandle) -> WalletHandle {
    trace!("set_wallet_handle >>> handle: {:?}", handle);
    unsafe { WALLET_HANDLE = handle; }
    global::agency_client::get_agency_client_mut().unwrap().set_wallet_handle(handle.0);
    unsafe { WALLET_HANDLE }
}

pub fn get_main_wallet_handle() -> WalletHandle { unsafe { WALLET_HANDLE } }

pub fn reset_main_wallet_handle() -> VcxResult<()> {
    set_wallet_handle(INVALID_WALLET_HANDLE);
    Ok(())
}

pub async fn create_main_wallet(config: &WalletConfig) -> VcxResult<()> {
    let wh = create_and_open_as_main_wallet(&config).await?;
    trace!("Created wallet with handle {:?}", wh);

    // If MS is already in wallet then just continue
    anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).await.ok();

    close_main_wallet().await?;
    Ok(())
}

pub async fn export_main_wallet(path: &str, backup_key: &str) -> VcxResult<()> {
    let wallet_handle = get_main_wallet_handle();
    trace!("export >>> wallet_handle: {:?}, path: {:?}, backup_key: ****", wallet_handle, path);

    let export_config = json!({ "key": backup_key, "path": &path}).to_string();
    indy::wallet::export_wallet(wallet_handle, &export_config)
        .await
        .map_err(VcxError::from)
}

pub async fn create_and_open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    if settings::indy_mocks_enabled() {
        warn!("open_as_main_wallet ::: Indy mocks enabled, skipping opening main wallet.");
        return Ok(set_wallet_handle(WalletHandle(1)));
    }

    wallet::create_indy_wallet(&wallet_config).await?;
    open_as_main_wallet(&wallet_config).await
}

pub async fn close_main_wallet() -> VcxResult<()> {
    trace!("close_main_wallet >>>");
    if settings::indy_mocks_enabled() {
        warn!("close_main_wallet >>> Indy mocks enabled, skipping closing wallet");
        set_wallet_handle(INVALID_WALLET_HANDLE);
        return Ok(());
    }

    indy::wallet::close_wallet(get_main_wallet_handle())
        .await?;

    reset_main_wallet_handle()?;
    Ok(())
}

pub async fn open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    let handle = libindy::wallet::open_wallet(wallet_config).await?;
    set_wallet_handle(handle);
    Ok(handle)
}

#[cfg(feature = "test_utils")]
pub mod tests {
    use crate::global;
    use crate::global::settings;
    use crate::global::wallet::{add_main_wallet_record, close_main_wallet, create_and_open_as_main_wallet, export_main_wallet};
    use crate::libindy::utils::signus::main_wallet_create_and_store_my_did;
    use crate::utils::devsetup::TempFile;

    use crate::libindy::utils::wallet::*;

    fn _record() -> (&'static str, &'static str, &'static str) {
        ("type1", "id1", "value1")
    }

    pub async fn create_main_wallet_and_its_backup() -> (TempFile, String, WalletConfig) {
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
        let _handle = create_and_open_as_main_wallet(&wallet_config).await.unwrap();

        let (my_did, my_vk) = main_wallet_create_and_store_my_did(None, None).await.unwrap();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
        global::agency_client::get_agency_client_mut().unwrap().set_my_vk(&my_vk);

        let backup_key = settings::get_config_value(settings::CONFIG_WALLET_BACKUP_KEY).unwrap();

        let (type_, id, value) = _record();
        add_main_wallet_record(type_, id, value, None).await.unwrap();

        export_main_wallet(&export_file.path, &backup_key).await.unwrap();

        close_main_wallet().await.unwrap();

        (export_file, wallet_name.to_string(), wallet_config)
    }
}

pub async fn add_main_wallet_record(xtype: &str, id: &str, value: &str, tags: Option<&str>) -> VcxResult<()> {
    trace!("add_record >>> xtype: {}, id: {}, value: {}, tags: {:?}", secret!(&xtype), secret!(&id), secret!(&value), secret!(&tags));

    if settings::indy_mocks_enabled() { return Ok(()); }

    indy::wallet::add_wallet_record(get_main_wallet_handle(), xtype, id, value, tags)
        .await
        .map_err(VcxError::from)
}

pub async fn get_main_wallet_record(xtype: &str, id: &str, options: &str) -> VcxResult<String> {
    trace!("get_record >>> xtype: {}, id: {}, options: {}", secret!(&xtype), secret!(&id), options);

    if settings::indy_mocks_enabled() {
        return Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string());
    }

    indy::wallet::get_wallet_record(get_main_wallet_handle(), xtype, id, options)
        .await
        .map_err(VcxError::from)
}

pub async fn delete_main_wallet_record(xtype: &str, id: &str) -> VcxResult<()> {
    trace!("delete_record >>> xtype: {}, id: {}", secret!(&xtype), secret!(&id));

    if settings::indy_mocks_enabled() { return Ok(()); }

    indy::wallet::delete_wallet_record(get_main_wallet_handle(), xtype, id)
        .await
        .map_err(VcxError::from)
}


pub async fn update_main_wallet_record_value(xtype: &str, id: &str, value: &str) -> VcxResult<()> {
    trace!("update_record_value >>> xtype: {}, id: {}, value: {}", secret!(&xtype), secret!(&id), secret!(&value));

    if settings::indy_mocks_enabled() { return Ok(()); }

    indy::wallet::update_wallet_record_value(get_main_wallet_handle(), xtype, id, value)
        .await
        .map_err(VcxError::from)
}

pub async fn add_main_wallet_record_tags(xtype: &str, id: &str, tags: &str) -> VcxResult<()> {
    trace!("add_record_tags >>> xtype: {}, id: {}, tags: {:?}", secret!(&xtype), secret!(&id), secret!(&tags));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    indy::wallet::add_wallet_record_tags(get_main_wallet_handle(), xtype, id, tags)
        .await
        .map_err(VcxError::from)
}

pub async fn update_main_wallet_record_tags(xtype: &str, id: &str, tags: &str) -> VcxResult<()> {
    trace!("update_record_tags >>> xtype: {}, id: {}, tags: {}", secret!(&xtype), secret!(&id), secret!(&tags));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    indy::wallet::update_wallet_record_tags(get_main_wallet_handle(), xtype, id, tags)
        .await
        .map_err(VcxError::from)
}

pub async fn delete_main_wallet_record_tags(xtype: &str, id: &str, tag_names: &str) -> VcxResult<()> {
    trace!("delete_record_tags >>> xtype: {}, id: {}, tag_names: {}", secret!(&xtype), secret!(&id), secret!(&tag_names));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    indy::wallet::delete_wallet_record_tags(get_main_wallet_handle(), xtype, id, tag_names)
        .await
        .map_err(VcxError::from)
}

pub async fn open_search_main_wallet(xtype: &str, query: &str, options: &str) -> VcxResult<SearchHandle> {
    trace!("open_search >>> xtype: {}, query: {}, options: {}", secret!(&xtype), query, options);

    if settings::indy_mocks_enabled() {
        return Ok(1);
    }

    indy::wallet::open_wallet_search(get_main_wallet_handle(), xtype, query, options)
        .await
        .map_err(VcxError::from)
}

pub async fn fetch_next_records_main_wallet(search_handle: SearchHandle, count: usize) -> VcxResult<String> {
    trace!("fetch_next_records >>> search_handle: {}, count: {}", search_handle, count);

    if settings::indy_mocks_enabled() {
        return Ok(String::from("{}"));
    }

    indy::wallet::fetch_wallet_search_next_records(get_main_wallet_handle(), search_handle, count)
        .await
        .map_err(VcxError::from)
}

pub async fn close_search_main_wallet(search_handle: SearchHandle) -> VcxResult<()> {
    trace!("close_search >>> search_handle: {}", search_handle);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    indy::wallet::close_wallet_search(search_handle)
        .await
        .map_err(VcxError::from)
}

pub async fn main_wallet_configure_issuer(enterprise_seed: &str) -> VcxResult<IssuerConfig> {
    let (institution_did, _institution_verkey) = signus::main_wallet_create_and_store_my_did(Some(enterprise_seed), None).await?;
    Ok(IssuerConfig {
        institution_did,
    })
}
