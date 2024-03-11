use std::sync::{Arc, RwLock};

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
    wallet::{
        askar::{
            askar_import_config::AskarImportConfig, askar_wallet_config::AskarWalletConfig,
            AskarWallet,
        },
        base_wallet::ManageWallet,
    },
};

use crate::{
    api_vcx::api_global::{
        profile::{get_main_anoncreds, try_get_main_wallet},
        wallet::{get_main_wallet, setup_global_anoncreds, BaseWallet},
    },
    errors::{
        error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
        mapping_from_ariesvcxcore::map_ariesvcx_core_result,
    },
};

pub static GLOBAL_ASKAR_WALLET: RwLock<Option<Arc<AskarWallet>>> = RwLock::new(None);

fn setup_global_wallet(wallet: Arc<AskarWallet>) -> LibvcxResult<()> {
    let mut b_wallet = GLOBAL_ASKAR_WALLET.write()?;
    *b_wallet = Some(wallet);

    setup_global_anoncreds()
}

pub async fn open_as_main_wallet(
    wallet_config: &AskarWalletConfig,
) -> LibvcxResult<Arc<AskarWallet>> {
    let wallet = Arc::new(wallet_config.open_wallet().await?);
    setup_global_wallet(wallet.clone())?;
    Ok(wallet)
}

pub async fn create_and_open_as_main_wallet(
    wallet_config: &AskarWalletConfig,
) -> LibvcxResult<Arc<impl BaseWallet>> {
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
            let mut b_wallet = GLOBAL_ASKAR_WALLET.write()?;
            *b_wallet = None;
        }
    }
    Ok(())
}

pub async fn create_main_wallet(config: &AskarWalletConfig) -> LibvcxResult<()> {
    create_and_open_as_main_wallet(config).await?;
    let wallet = get_main_wallet()?;

    // If MS is already in wallet then just continue
    get_main_anoncreds()?
        .prover_create_link_secret(wallet.as_ref(), &DEFAULT_LINK_SECRET_ALIAS.to_string())
        .await
        .ok();

    // TODO: enable when closing askar wallet is implemented
    // close_main_wallet().await?;
    Ok(())
}

pub async fn wallet_import(config: &AskarImportConfig) -> LibvcxResult<()> {
    map_ariesvcx_core_result(config.import_wallet().await)
}

pub async fn wallet_migrate(wallet_config: &impl ManageWallet) -> LibvcxResult<()> {
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
    use aries_vcx_core::wallet::askar::{
        askar_wallet_config::AskarWalletConfig, key_method::KeyMethod,
    };
    use test_utils::devsetup::TempFile;
    use uuid::Uuid;

    // use crate::api_vcx::api_global::wallet::askar::open_main_wallet;
    use crate::api_vcx::api_global::wallet::askar::open_as_main_wallet;
    use crate::{
        api_vcx::api_global::wallet::{
            askar::{create_and_open_as_main_wallet, create_main_wallet},
            test_utils::setup_wallet_backup,
        },
        errors::error::LibvcxResult,
    };

    pub async fn _create_main_wallet_and_its_backup() -> (TempFile, String, AskarWalletConfig) {
        let wallet_config = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );

        let wallet_name = &format!("export_test_wallet_{}", uuid::Uuid::new_v4());

        let export_file = TempFile::prepare_path(wallet_name);

        let wallet = create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();

        setup_wallet_backup(wallet.as_ref(), &export_file).await;

        (export_file, wallet_name.to_string(), wallet_config)
    }

    pub async fn _create_and_open_wallet() -> LibvcxResult<AskarWalletConfig> {
        let config_wallet: AskarWalletConfig = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );
        create_main_wallet(&config_wallet).await?;
        open_as_main_wallet(&config_wallet).await?;
        Ok(config_wallet)
    }
}

#[cfg(test)]
pub mod tests {
    use test_utils::devsetup::SetupMocks;

    use crate::api_vcx::api_global::wallet::askar::create_main_wallet;

    #[tokio::test]
    async fn test_wallet_migrate() {
        use aries_vcx_core::wallet::askar::{
            askar_wallet_config::AskarWalletConfig, key_method::KeyMethod,
        };
        use uuid::Uuid;

        use crate::api_vcx::api_global::wallet::askar::create_and_open_as_main_wallet;

        let config = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );

        create_and_open_as_main_wallet(&config).await.unwrap();

        let new_config = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );

        super::wallet_migrate(&new_config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_create() {
        use aries_vcx_core::wallet::askar::{
            askar_wallet_config::AskarWalletConfig, key_method::KeyMethod,
        };
        use uuid::Uuid;

        let _setup = SetupMocks::init();

        let config = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );

        create_main_wallet(&config).await.unwrap();
    }
}
