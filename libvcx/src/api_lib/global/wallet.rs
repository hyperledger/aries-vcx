use aries_vcx::error::VcxResult;
use aries_vcx::global::settings;
use aries_vcx::vdrtools::{INVALID_WALLET_HANDLE, WalletHandle};
use aries_vcx::libindy;
use aries_vcx::libindy::wallet::WalletConfig;
use aries_vcx::libindy::{anoncreds, wallet};

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

pub fn reset_main_wallet_handle() -> VcxResult<()> {
    set_main_wallet_handle(INVALID_WALLET_HANDLE);
    Ok(())
}

pub async fn export_main_wallet(path: &str, backup_key: &str) -> VcxResult<()> {
    libindy::wallet::export_wallet(get_main_wallet_handle(), path, backup_key).await
}

pub async fn open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    let handle = libindy::wallet::open_wallet(wallet_config).await?;
    set_main_wallet_handle(handle);
    Ok(handle)
}

pub async fn create_and_open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    let handle = libindy::wallet::create_and_open_wallet(wallet_config).await?;
    set_main_wallet_handle(handle);
    Ok(handle)
}

pub async fn close_main_wallet() -> VcxResult<()> {
    libindy::wallet::close_wallet(get_main_wallet_handle()).await?;
    reset_main_wallet_handle()?;
    Ok(())
}

pub async fn create_main_wallet(config: &WalletConfig) -> VcxResult<()> {
    let wallet_handle = create_and_open_as_main_wallet(&config).await?;
    trace!("Created wallet with handle {:?}", wallet_handle);

    // If MS is already in wallet then just continue
    anoncreds::libindy_prover_create_master_secret(wallet_handle, settings::DEFAULT_LINK_SECRET_ALIAS)
        .await
        .ok();

    close_main_wallet().await?;
    Ok(())
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use aries_vcx::global;
    use aries_vcx::global::settings;
    use aries_vcx::libindy::signus::create_and_store_my_did;
    use aries_vcx::libindy::wallet::{add_wallet_record, WalletConfig};
    use aries_vcx::libindy::wallet::*;
    use aries_vcx::utils::devsetup::TempFile;

    use crate::api_lib::global::wallet::{close_main_wallet, create_and_open_as_main_wallet, export_main_wallet};

    fn _record() -> (&'static str, &'static str, &'static str) {
        ("type1", "id1", "value1")
    }

    pub async fn _create_main_wallet_and_its_backup() -> (TempFile, String, WalletConfig) {
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
        create_and_store_my_did(wallet_handle, None, None).await.unwrap();
        let backup_key = settings::get_config_value(settings::CONFIG_WALLET_BACKUP_KEY).unwrap();
        let (type_, id, value) = _record();
        add_wallet_record(wallet_handle, type_, id, value, None).await.unwrap();
        export_main_wallet(&export_file.path, &backup_key).await.unwrap();

        close_main_wallet().await.unwrap();

        // todo: import and verify
        (export_file, wallet_name.to_string(), wallet_config)
    }
}
