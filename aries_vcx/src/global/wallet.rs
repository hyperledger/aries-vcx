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
    let wallet_handle = create_and_open_as_main_wallet(&config).await?;
    trace!("Created wallet with handle {:?}", wallet_handle);

    // If MS is already in wallet then just continue
    anoncreds::libindy_prover_create_master_secret(wallet_handle, settings::DEFAULT_LINK_SECRET_ALIAS).await.ok();

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

pub async fn main_wallet_configure_issuer(enterprise_seed: &str) -> VcxResult<IssuerConfig> {
    let (institution_did, _institution_verkey) = signus::create_and_store_my_did(get_main_wallet_handle(), Some(enterprise_seed), None).await?;
    Ok(IssuerConfig {
        institution_did,
    })
}


#[cfg(feature = "test_utils")]
pub mod tests {
    use crate::global;
    use crate::global::settings;
    use crate::global::wallet::{close_main_wallet, create_and_open_as_main_wallet, export_main_wallet};
    use crate::libindy::utils::signus::create_and_store_my_did;
    use crate::utils::devsetup::TempFile;

    use crate::libindy::utils::wallet::*;
    use crate::libindy::utils::wallet::add_main_wallet_record;

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
        let wallet_handle = create_and_open_as_main_wallet(&wallet_config).await.unwrap();

        let (my_did, my_vk) = create_and_store_my_did(wallet_handle, None, None).await.unwrap();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
        global::agency_client::get_agency_client_mut().unwrap().set_my_vk(&my_vk);

        let backup_key = settings::get_config_value(settings::CONFIG_WALLET_BACKUP_KEY).unwrap();

        let (type_, id, value) = _record();
        add_main_wallet_record(wallet_handle, type_, id, value, None).await.unwrap();

        export_main_wallet(&export_file.path, &backup_key).await.unwrap();

        close_main_wallet().await.unwrap();

        (export_file, wallet_name.to_string(), wallet_config)
    }
}