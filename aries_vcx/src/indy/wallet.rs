use vdrtools::{
    types::{
        domain::wallet::{default_key_derivation_method, KeyDerivationMethod},
        errors::IndyErrorKind,
    },
    Locator, SearchHandle, WalletHandle,
};

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings,
    indy::{credentials::holder, keys},
};

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct WalletConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_key_derivation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_credentials: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_derivation_method: Option<String>,
}

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct IssuerConfig {
    pub institution_did: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WalletCredentials {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_credentials: Option<serde_json::Value>,
    key_derivation_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rekey_derivation_method: Option<String>,
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

pub async fn open_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    trace!("open_as_main_wallet >>> {}", &wallet_config.wallet_name);

    let handle_res = Locator::instance()
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
        .await;

    Ok(handle_res?)
}

fn parse_key_derivation_method(method: &str) -> Result<KeyDerivationMethod, AriesVcxError> {
    match method {
        "RAW" => Ok(KeyDerivationMethod::RAW),
        "ARGON2I_MOD" => Ok(KeyDerivationMethod::ARGON2I_MOD),
        "ARGON2I_INT" => Ok(KeyDerivationMethod::ARGON2I_INT),
        _ => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidOption,
            format!("Unknown derivation method {}", method),
        )),
    }
}

pub(crate) async fn create_indy_wallet(wallet_config: &WalletConfig) -> VcxResult<()> {
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

        Err(err) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::WalletCreate,
            format!("could not create wallet {}: {}", wallet_config.wallet_name, err,),
        )),
    }
}

pub async fn delete_wallet(wallet_config: &WalletConfig) -> VcxResult<()> {
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

        Err(err) if err.kind() == IndyErrorKind::WalletAccessFailed => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::WalletAccessFailed,
            format!(
                "Can not open wallet \"{}\". Invalid key has been provided.",
                &wallet_config.wallet_name
            ),
        )),

        Err(err) if err.kind() == IndyErrorKind::WalletNotFound => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::WalletNotFound,
            format!("Wallet \"{}\" not found or unavailable", &wallet_config.wallet_name,),
        )),

        Err(err) => Err(err.into()),
    }
}

pub async fn import(restore_config: &RestoreWalletConfigs) -> VcxResult<()> {
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

pub(crate) async fn add_wallet_record(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    value: &str,
    tags: Option<&str>,
) -> VcxResult<()> {
    trace!(
        "add_record >>> xtype: {}, id: {}, value: {}, tags: {:?}",
        secret!(&xtype),
        secret!(&id),
        secret!(&value),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .add_record(
            wallet_handle,
            xtype.into(),
            id.into(),
            value.into(),
            tags.map(serde_json::from_str).transpose()?,
        )
        .await?;

    Ok(())
}

pub(crate) async fn get_wallet_record(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    options: &str,
) -> VcxResult<String> {
    trace!(
        "get_record >>> xtype: {}, id: {}, options: {}",
        secret!(&xtype),
        secret!(&id),
        options
    );

    if settings::indy_mocks_enabled() {
        return Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string());
    }

    let res = Locator::instance()
        .non_secret_controller
        .get_record(wallet_handle, xtype.into(), id.into(), options.into())
        .await?;

    Ok(res)
}

pub(crate) async fn delete_wallet_record(wallet_handle: WalletHandle, xtype: &str, id: &str) -> VcxResult<()> {
    trace!("delete_record >>> xtype: {}, id: {}", secret!(&xtype), secret!(&id));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .delete_record(wallet_handle, xtype.into(), id.into())
        .await?;

    Ok(())
}

pub(crate) async fn update_wallet_record_value(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    value: &str,
) -> VcxResult<()> {
    trace!(
        "update_record_value >>> xtype: {}, id: {}, value: {}",
        secret!(&xtype),
        secret!(&id),
        secret!(&value)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .update_record_value(wallet_handle, xtype.into(), id.into(), value.into())
        .await?;

    Ok(())
}

pub(crate) async fn add_wallet_record_tags(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    tags: &str,
) -> VcxResult<()> {
    trace!(
        "add_record_tags >>> xtype: {}, id: {}, tags: {:?}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .add_record_tags(wallet_handle, xtype.into(), id.into(), serde_json::from_str(tags)?)
        .await?;

    Ok(())
}

pub(crate) async fn update_wallet_record_tags(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    tags: &str,
) -> VcxResult<()> {
    trace!(
        "update_record_tags >>> xtype: {}, id: {}, tags: {}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .update_record_tags(wallet_handle, xtype.into(), id.into(), serde_json::from_str(tags)?)
        .await?;

    Ok(())
}

pub(crate) async fn delete_wallet_record_tags(
    wallet_handle: WalletHandle,
    xtype: &str,
    id: &str,
    tag_names: &str,
) -> VcxResult<()> {
    trace!(
        "delete_record_tags >>> xtype: {}, id: {}, tag_names: {}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tag_names)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .delete_record_tags(wallet_handle, xtype.into(), id.into(), tag_names.into())
        .await?;

    Ok(())
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn open_search_wallet(
    wallet_handle: WalletHandle,
    xtype: &str,
    query: &str,
    options: &str,
) -> VcxResult<SearchHandle> {
    trace!(
        "open_search >>> xtype: {}, query: {}, options: {}",
        secret!(&xtype),
        query,
        options
    );

    if settings::indy_mocks_enabled() {
        return Ok(SearchHandle(1));
    }

    let res = Locator::instance()
        .non_secret_controller
        .open_search(wallet_handle, xtype.into(), query.into(), options.into())
        .await?;

    Ok(res)
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn fetch_next_records_wallet(
    wallet_handle: WalletHandle,
    search_handle: SearchHandle,
    count: usize,
) -> VcxResult<String> {
    trace!(
        "fetch_next_records >>> search_handle: {}, count: {}",
        search_handle.0,
        count
    );

    if settings::indy_mocks_enabled() {
        return Ok(String::from("{}"));
    }

    let res = Locator::instance()
        .non_secret_controller
        .fetch_search_next_records(wallet_handle, search_handle, count)
        .await?;

    Ok(res)
}

// TODO - FUTURE - revert to pub(crate) after libvcx dependency is fixed
pub async fn close_search_wallet(search_handle: SearchHandle) -> VcxResult<()> {
    trace!("close_search >>> search_handle: {:?}", search_handle);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    Locator::instance()
        .non_secret_controller
        .close_search(search_handle)
        .await?;

    Ok(())
}

// TODO - FUTURE - can this be moved externally - move to a generic setup util?
pub async fn wallet_configure_issuer(wallet_handle: WalletHandle, enterprise_seed: &str) -> VcxResult<IssuerConfig> {
    let (institution_did, _institution_verkey) =
        keys::create_and_store_my_did(wallet_handle, Some(enterprise_seed), None).await?;

    Ok(IssuerConfig { institution_did })
}

pub async fn create_wallet_with_master_secret(config: &WalletConfig) -> VcxResult<()> {
    let wallet_handle = create_and_open_wallet(config).await?;

    trace!("Created wallet with handle {:?}", wallet_handle);

    // If MS is already in wallet then just continue
    holder::libindy_prover_create_master_secret(wallet_handle, settings::DEFAULT_LINK_SECRET_ALIAS)
        .await
        .ok();

    Locator::instance().wallet_controller.close(wallet_handle).await?;

    Ok(())
}

pub async fn export_wallet(wallet_handle: WalletHandle, path: &str, backup_key: &str) -> VcxResult<()> {
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

pub async fn create_and_open_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    if settings::indy_mocks_enabled() {
        warn!("create_and_open_wallet ::: Indy mocks enabled, skipping opening main wallet.");
        return Ok(WalletHandle(1));
    }

    create_indy_wallet(wallet_config).await?;

    let handle = open_wallet(wallet_config).await?;

    Ok(handle)
}

pub async fn close_wallet(wallet_handle: WalletHandle) -> VcxResult<()> {
    trace!("close_wallet >>>");

    if settings::indy_mocks_enabled() {
        warn!("close_wallet >>> Indy mocks enabled, skipping closing wallet");
        return Ok(());
    }

    Locator::instance().wallet_controller.close(wallet_handle).await?;

    Ok(())
}

#[cfg(feature = "general_test")]
#[cfg(test)]
mod test {
    use crate::{
        errors::error::AriesVcxErrorKind, indy::wallet::add_wallet_record, utils::devsetup::SetupLibraryWallet,
    };

    #[tokio::test]
    async fn test_add_record() {
        SetupLibraryWallet::run(|setup| async move {
            add_wallet_record(setup.wallet_handle, "record_type", "123", "Record Value", Some("{}"))
                .await
                .unwrap();
            let err = add_wallet_record(setup.wallet_handle, "record_type", "123", "Record Value", Some("{}"))
                .await
                .unwrap_err();
            assert_eq!(err.kind(), AriesVcxErrorKind::DuplicationWalletRecord);
        })
        .await;
    }
}
