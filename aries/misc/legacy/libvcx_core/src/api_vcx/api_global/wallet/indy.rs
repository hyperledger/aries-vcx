use std::sync::{Arc, RwLock};

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
    wallet::{
        base_wallet::{BaseWallet, ImportWallet, ManageWallet},
        indy::{
            indy_import_config::IndyImportConfig, indy_wallet_config::IndyWalletConfig,
            IndySdkWallet,
        },
    },
};

use crate::{
    api_vcx::api_global::{
        profile::{get_main_anoncreds, try_get_main_wallet},
        wallet::{get_main_wallet, setup_global_anoncreds},
    },
    errors::{
        error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
        mapping_from_ariesvcxcore::map_ariesvcx_core_result,
    },
};

pub static GLOBAL_INDY_WALLET: RwLock<Option<Arc<IndySdkWallet>>> = RwLock::new(None);

fn setup_global_wallet(wallet: Arc<IndySdkWallet>) -> LibvcxResult<()> {
    let mut b_wallet = GLOBAL_INDY_WALLET.write()?;
    *b_wallet = Some(wallet);

    setup_global_anoncreds()
}

pub async fn open_as_main_wallet(
    wallet_config: &IndyWalletConfig,
) -> LibvcxResult<Arc<IndySdkWallet>> {
    let wallet = Arc::new(wallet_config.open_wallet().await?);
    setup_global_wallet(wallet.clone())?;
    Ok(wallet)
}

pub async fn create_and_open_as_main_wallet(
    wallet_config: &IndyWalletConfig,
) -> LibvcxResult<Arc<IndySdkWallet>> {
    let wallet = Arc::new(wallet_config.create_wallet().await?);

    setup_global_wallet(wallet.clone())?;
    Ok(wallet)
}

pub async fn close_main_wallet() -> LibvcxResult<()> {
    let wallet = try_get_main_wallet()?;
    match wallet {
        None => {
            warn!("Skipping wallet close, no global wallet component available.")
        }
        Some(wallet) => {
            wallet.close_wallet().await?;
            let mut b_wallet = GLOBAL_INDY_WALLET.write()?;
            *b_wallet = None;
        }
    }
    Ok(())
}

pub async fn create_main_wallet(config: &IndyWalletConfig) -> LibvcxResult<()> {
    let wallet = create_and_open_as_main_wallet(config).await?;
    trace!("Created wallet {:?}", wallet);
    let wallet = get_main_wallet()?;

    // If MS is already in wallet then just continue
    get_main_anoncreds()?
        .prover_create_link_secret(wallet.as_ref(), &DEFAULT_LINK_SECRET_ALIAS.to_string())
        .await
        .ok();

    close_main_wallet().await?;
    Ok(())
}

pub async fn wallet_import(config: &IndyImportConfig) -> LibvcxResult<()> {
    map_ariesvcx_core_result(config.import_wallet().await)
}

pub async fn wallet_migrate(wallet_config: &IndyWalletConfig) -> LibvcxResult<()> {
    let src_wallet = get_main_wallet()?;
    info!("Opening target wallet.");
    let dest_wallet = wallet_config.create_wallet().await?;

    let migration_res = wallet_migrator::migrate_wallet(
        src_wallet.as_ref(),
        &dest_wallet,
        wallet_migrator::vdrtools2credx::migrate_any_record,
    )
    .await;

    migration_res.map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::WalletMigrationFailed, e))
}

#[cfg(test)]
pub mod test_utils {
    use aries_vcx_core::global::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW};
    use test_utils::devsetup::TempFile;

    use crate::{
        api_vcx::api_global::wallet::{
            indy::{
                create_and_open_as_main_wallet, create_main_wallet, open_as_main_wallet,
                IndyWalletConfig,
            },
            test_utils::setup_wallet_backup,
        },
        errors::error::LibvcxResult,
    };

    pub async fn _create_and_open_wallet() -> LibvcxResult<IndyWalletConfig> {
        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());
        let config_wallet: IndyWalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))?;
        create_main_wallet(&config_wallet).await?;
        open_as_main_wallet(&config_wallet).await?;
        Ok(config_wallet)
    }

    pub async fn _create_main_wallet_and_its_backup() -> (TempFile, String, IndyWalletConfig) {
        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());

        let wallet_config = IndyWalletConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let export_file = TempFile::prepare_path(&wallet_name);

        let wallet = create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();

        setup_wallet_backup(wallet.as_ref(), &export_file).await;

        (export_file, wallet_name.to_string(), wallet_config)
    }
}

#[cfg(test)]
pub mod tests {
    use aries_vcx_core::{
        global::settings::{DEFAULT_WALLET_BACKUP_KEY, DEFAULT_WALLET_KEY, WALLET_KDF_RAW},
        wallet::base_wallet::ManageWallet,
    };
    use test_utils::devsetup::{SetupMocks, TempFile};

    use crate::{
        api_vcx::api_global::wallet::{
            export_main_wallet,
            indy::{
                close_main_wallet, create_and_open_as_main_wallet, create_main_wallet,
                open_as_main_wallet, test_utils::_create_main_wallet_and_its_backup, wallet_import,
                IndyImportConfig, IndyWalletConfig,
            },
        },
        errors::error::LibvcxErrorKind,
    };

    #[tokio::test]
    async fn test_wallet_create() {
        let _setup = SetupMocks::init();

        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());
        let config: IndyWalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();

        create_main_wallet(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_migrate() {
        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());
        let config: IndyWalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();

        create_and_open_as_main_wallet(&config).await.unwrap();

        let wallet_name = format!("test_migrate_wallet_{}", uuid::Uuid::new_v4());
        let new_config: IndyWalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();

        super::wallet_migrate(&new_config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_export_import() {
        let _setup = SetupMocks::init();
        let wallet_name = uuid::Uuid::new_v4().to_string();
        let export_file = TempFile::prepare_path(&wallet_name);
        let wallet_config = IndyWalletConfig {
            wallet_name,
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();
        let backup_key = DEFAULT_WALLET_BACKUP_KEY;
        export_main_wallet(&export_file.path.to_string(), backup_key)
            .await
            .unwrap();
        close_main_wallet().await.unwrap();
        wallet_config.delete_wallet().await.unwrap();
        let import_config: IndyImportConfig = serde_json::from_value(json!({
            "wallet_name": wallet_config.wallet_name.clone(),
            "wallet_key": wallet_config.wallet_key.clone(),
            "exported_wallet_path": export_file.path,
            "backup_key": backup_key,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();
        wallet_import(&import_config).await.unwrap();
        wallet_config.delete_wallet().await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_open_with_incorrect_key_fails() {
        let _setup = SetupMocks::init();
        let wallet_name = uuid::Uuid::new_v4().to_string();
        let _export_file = TempFile::prepare_path(&wallet_name);
        let mut wallet_config = IndyWalletConfig {
            wallet_name,
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();
        close_main_wallet().await.unwrap();
        wallet_config.wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFA2cAA".to_string();
        let err = open_as_main_wallet(&wallet_config).await.unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::WalletAccessFailed);
    }

    #[tokio::test]
    async fn test_wallet_open_with_wrong_name_fails() {
        let _setup = SetupMocks::init();

        let wallet_config: IndyWalletConfig = serde_json::from_value(json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW,
        }))
        .unwrap();

        assert_eq!(
            open_as_main_wallet(&wallet_config)
                .await
                .unwrap_err()
                .kind(),
            LibvcxErrorKind::WalletNotFound
        )
    }

    #[tokio::test]
    async fn test_wallet_open_of_imported_wallet_succeeds() {
        let _setup = SetupMocks::init();

        let (export_wallet_path, wallet_name, wallet_config) =
            _create_main_wallet_and_its_backup().await;

        wallet_config.delete_wallet().await.unwrap();

        let import_config = IndyImportConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: Some(WALLET_KDF_RAW.into()),
        };
        wallet_import(&import_config).await.unwrap();

        let wallet_config: IndyWalletConfig = serde_json::from_value(json!({
            "wallet_name": &wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW,
        }))
        .unwrap();

        open_as_main_wallet(&wallet_config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_import_of_opened_wallet_fails() {
        let _setup = SetupMocks::init();

        let (export_wallet_path, wallet_name, wallet_config) =
            _create_main_wallet_and_its_backup().await;

        open_as_main_wallet(&wallet_config).await.unwrap();

        let import_config = IndyImportConfig {
            wallet_name,
            wallet_key: DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: None,
        };
        assert_eq!(
            wallet_import(&import_config).await.unwrap_err().kind(),
            LibvcxErrorKind::DuplicationWallet
        )
    }
}
