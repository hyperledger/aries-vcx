use serde::{Deserialize, Serialize};

use vdrtools::{DidMethod, DidValue, KeyInfo, Locator, MyDidInfo, types::domain::wallet::{default_key_derivation_method, KeyDerivationMethod}, types::errors::IndyErrorKind};

use crate::{secret, utils};
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    indy::credentials::holder,
};
use crate::{global::settings, WalletHandle};
use crate::indy::utils::mocks::did_mocks::{did_mocks_enabled, DidMocks};
use crate::SearchHandle;
use crate::wallet::indy::{IssuerConfig, RestoreWalletConfigs, WalletConfig};

pub async fn open_wallet(wallet_config: &WalletConfig) -> VcxCoreResult<WalletHandle> {
    trace!("open_as_main_wallet >>> {}", &wallet_config.wallet_name);

    Locator::instance()
        .wallet_controller
        .open(
            vdrtools::types::domain::wallet::Config {
                id: wallet_config.wallet_name.clone(),
                storage_type: wallet_config.wallet_type.clone(),
                storage_config: wallet_config
                    .storage_config
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?,
                cache: None,
            },
            vdrtools::types::domain::wallet::Credentials {
                key: wallet_config.wallet_key.clone(),
                key_derivation_method: parse_key_derivation_method(&wallet_config.wallet_key_derivation)?,

                rekey: wallet_config.rekey.clone(),
                rekey_derivation_method: wallet_config
                    .rekey_derivation_method
                    .as_deref()
                    .map(parse_key_derivation_method)
                    .transpose()?
                    .unwrap_or_else(default_key_derivation_method),

                storage_credentials: wallet_config
                    .storage_credentials
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?,
            },
        )
        .await
        .map_err(From::from)
}

fn parse_key_derivation_method(method: &str) -> Result<KeyDerivationMethod, AriesVcxCoreError> {
    match method {
        "RAW" => Ok(KeyDerivationMethod::RAW),
        "ARGON2I_MOD" => Ok(KeyDerivationMethod::ARGON2I_MOD),
        "ARGON2I_INT" => Ok(KeyDerivationMethod::ARGON2I_INT),
        _ => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidOption,
            format!("Unknown derivation method {method}"),
        )),
    }
}

pub async fn create_indy_wallet(wallet_config: &WalletConfig) -> VcxCoreResult<()> {
    trace!("create_wallet >>> {}", &wallet_config.wallet_name);

    let credentials = vdrtools::types::domain::wallet::Credentials {
        key: wallet_config.wallet_key.clone(),
        key_derivation_method: parse_key_derivation_method(&wallet_config.wallet_key_derivation)?,

        rekey: None,
        rekey_derivation_method: default_key_derivation_method(),

        storage_credentials: wallet_config
            .storage_credentials
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
    };

    trace!("Credentials: {:?}", credentials);

    let res = Locator::instance()
        .wallet_controller
        .create(
            vdrtools::types::domain::wallet::Config {
                id: wallet_config.wallet_name.clone(),
                storage_type: wallet_config.wallet_type.clone(),
                storage_config: wallet_config
                    .storage_config
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?,
                cache: None,
            },
            credentials,
        )
        .await;

    match res {
        Ok(()) => Ok(()),

        Err(err) if err.kind() == IndyErrorKind::WalletAlreadyExists => {
            warn!(
                "wallet \"{}\" already exists. skipping creation",
                wallet_config.wallet_name
            );
            Ok(())
        }

        Err(err) => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::WalletCreate,
            format!("could not create wallet {}: {}", wallet_config.wallet_name, err, ),
        )),
    }
}

pub async fn delete_wallet(wallet_config: &WalletConfig) -> VcxCoreResult<()> {
    trace!("delete_wallet >>> wallet_name: {}", &wallet_config.wallet_name);

    let credentials = vdrtools::types::domain::wallet::Credentials {
        key: wallet_config.wallet_key.clone(),
        key_derivation_method: parse_key_derivation_method(&wallet_config.wallet_key_derivation)?,

        rekey: None,
        rekey_derivation_method: default_key_derivation_method(),

        storage_credentials: wallet_config
            .storage_credentials
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
    };

    trace!("Credentials: {:?}", credentials);

    let res = Locator::instance()
        .wallet_controller
        .delete(
            vdrtools::types::domain::wallet::Config {
                id: wallet_config.wallet_name.clone(),
                storage_type: wallet_config.wallet_type.clone(),
                storage_config: wallet_config
                    .storage_config
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()?,
                cache: None,
            },
            credentials,
        )
        .await;

    match res {
        Ok(_) => Ok(()),

        Err(err) if err.kind() == IndyErrorKind::WalletAccessFailed => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::WalletAccessFailed,
            format!(
                "Can not open wallet \"{}\". Invalid key has been provided.",
                &wallet_config.wallet_name
            ),
        )),

        Err(err) if err.kind() == IndyErrorKind::WalletNotFound => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::WalletNotFound,
            format!("Wallet \"{}\" not found or unavailable", &wallet_config.wallet_name, ),
        )),

        Err(err) => Err(err.into()),
    }
}

pub async fn import(restore_config: &RestoreWalletConfigs) -> VcxCoreResult<()> {
    trace!(
        "import >>> wallet: {} exported_wallet_path: {}",
        restore_config.wallet_name,
        restore_config.exported_wallet_path
    );

    Locator::instance()
        .wallet_controller
        .import(
            vdrtools::types::domain::wallet::Config {
                id: restore_config.wallet_name.clone(),
                ..Default::default()
            },
            vdrtools::types::domain::wallet::Credentials {
                key: restore_config.wallet_key.clone(),
                key_derivation_method: restore_config
                    .wallet_key_derivation
                    .as_deref()
                    .map(parse_key_derivation_method)
                    .transpose()?
                    .unwrap_or_else(default_key_derivation_method),

                rekey: None,
                rekey_derivation_method: default_key_derivation_method(), // default value

                storage_credentials: None, // default value
            },
            vdrtools::types::domain::wallet::ExportConfig {
                key: restore_config.backup_key.clone(),
                path: restore_config.exported_wallet_path.clone(),

                key_derivation_method: default_key_derivation_method(),
            },
        )
        .await?;

    Ok(())
}

// TODO - FUTURE - can this be moved externally - move to a generic setup util?
pub async fn wallet_configure_issuer(
    wallet_handle: WalletHandle,
    enterprise_seed: &str,
) -> VcxCoreResult<IssuerConfig> {
    let (institution_did, _institution_verkey) =
        create_and_store_my_did(wallet_handle, Some(enterprise_seed), None).await?;

    Ok(IssuerConfig { institution_did })
}

pub async fn create_wallet_with_master_secret(config: &WalletConfig) -> VcxCoreResult<()> {
    let wallet_handle = create_and_open_wallet(config).await?;

    trace!("Created wallet with handle {:?}", wallet_handle);

    // If MS is already in wallet then just continue
    holder::libindy_prover_create_master_secret(wallet_handle, settings::DEFAULT_LINK_SECRET_ALIAS)
        .await
        .ok();

    Locator::instance().wallet_controller.close(wallet_handle).await?;

    Ok(())
}

pub async fn export_wallet(wallet_handle: WalletHandle, path: &str, backup_key: &str) -> VcxCoreResult<()> {
    trace!(
        "export >>> wallet_handle: {:?}, path: {:?}, backup_key: ****",
        wallet_handle,
        path
    );

    Locator::instance()
        .wallet_controller
        .export(
            wallet_handle,
            vdrtools::types::domain::wallet::ExportConfig {
                key: backup_key.into(),
                path: path.into(),

                key_derivation_method: default_key_derivation_method(),
            },
        )
        .await?;

    Ok(())
}

pub async fn create_and_open_wallet(wallet_config: &WalletConfig) -> VcxCoreResult<WalletHandle> {
    if settings::indy_mocks_enabled() {
        warn!("create_and_open_wallet ::: Indy mocks enabled, skipping opening main wallet.");
        return Ok(WalletHandle(0));
    }

    create_indy_wallet(wallet_config).await?;

    let handle = open_wallet(wallet_config).await?;

    Ok(handle)
}

pub async fn close_wallet(wallet_handle: WalletHandle) -> VcxCoreResult<()> {
    trace!("close_wallet >>>");

    if settings::indy_mocks_enabled() {
        warn!("close_wallet >>> Indy mocks enabled, skipping closing wallet");
        return Ok(());
    }

    Locator::instance().wallet_controller.close(wallet_handle).await?;

    Ok(())
}

pub async fn create_and_store_my_did(
    wallet_handle: WalletHandle,
    seed: Option<&str>,
    method_name: Option<&str>,
) -> VcxCoreResult<(String, String)> {
    trace!(
        "create_and_store_my_did >>> seed: {:?}, method_name: {:?}",
        seed,
        method_name
    );

    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()));
    }

    let res = Locator::instance()
        .did_controller
        .create_and_store_my_did(
            wallet_handle,
            MyDidInfo {
                method_name: method_name.map(|m| DidMethod(m.into())),
                seed: seed.map(ToOwned::to_owned),
                ..MyDidInfo::default()
            },
        )
        .await?;

    Ok(res)
}

pub async fn libindy_replace_keys_start(wallet_handle: WalletHandle, did: &str) -> VcxCoreResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("libindy_replace_keys_start >> retrieving did mock response");
        return Ok(DidMocks::get_next_did_response());
    }

    let res = Locator::instance()
        .did_controller
        .replace_keys_start(wallet_handle, KeyInfo::default(), DidValue(did.into()))
        .await?;

    Ok(res)
}

pub async fn libindy_replace_keys_apply(wallet_handle: WalletHandle, did: &str) -> VcxCoreResult<()> {
    if did_mocks_enabled() {
        warn!("libindy_replace_keys_apply >> retrieving did mock response");
        return Ok(());
    }

    Locator::instance()
        .did_controller
        .replace_keys_apply(wallet_handle, DidValue(did.into()))
        .await?;

    Ok(())
}

pub async fn get_verkey_from_wallet(wallet_handle: WalletHandle, did: &str) -> VcxCoreResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("get_verkey_from_wallet >> retrieving did mock response");
        return Ok(DidMocks::get_next_did_response());
    }

    let res = Locator::instance()
        .did_controller
        .key_for_local_did(wallet_handle, DidValue(did.into()))
        .await?;

    Ok(res)
}
