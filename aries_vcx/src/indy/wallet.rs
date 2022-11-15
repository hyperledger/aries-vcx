use vdrtools::wallet;
use vdrtools_sys::SearchHandle;
use crate::error::{VcxError, VcxErrorExt, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::indy::keys;
use crate::indy::credentials::holder;
use crate::vdrtools::{ErrorCode, WalletHandle};

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

impl RestoreWalletConfigs {
    pub fn from_str(data: &str) -> VcxResult<RestoreWalletConfigs> {
        serde_json::from_str(data).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize RestoreWalletConfigs: {:?}", err),
            )
        })
    }
}


pub async fn open_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    trace!("open_as_main_wallet >>> {}", &wallet_config.wallet_name);
    let config = build_wallet_config(
        &wallet_config.wallet_name,
        wallet_config.wallet_type.as_deref(),
        wallet_config.storage_config.as_deref(),
    );
    let credentials = build_wallet_credentials(
        &wallet_config.wallet_key,
        wallet_config.storage_credentials.as_deref(),
        &wallet_config.wallet_key_derivation,
        wallet_config.rekey.as_deref(),
        wallet_config.rekey_derivation_method.as_deref(),
    )?;

    let handle = vdrtools::wallet::open_wallet(&config, &credentials)
        .await
        .map_err(|err| match err.error_code {
            ErrorCode::WalletAlreadyOpenedError => err.to_vcx(
                VcxErrorKind::WalletAlreadyOpen,
                format!("Wallet \"{}\" already opened.", wallet_config.wallet_name),
            ),
            ErrorCode::WalletAccessFailed => err.to_vcx(
                VcxErrorKind::WalletAccessFailed,
                format!(
                    "Can not open wallet \"{}\". Invalid key has been provided.",
                    wallet_config.wallet_name
                ),
            ),
            ErrorCode::WalletNotFoundError => err.to_vcx(
                VcxErrorKind::WalletNotFound,
                format!("Wallet \"{}\" not found or unavailable", wallet_config.wallet_name),
            ),
            error_code => err.to_vcx(VcxErrorKind::LibndyError(error_code as u32), "Indy error occurred"),
        })?;

    Ok(handle)
}

pub(crate) fn build_wallet_config(wallet_name: &str, wallet_type: Option<&str>, storage_config: Option<&str>) -> String {
    let mut config = json!({
        "id": wallet_name,
        "storage_type": wallet_type
    });
    if let Some(_config) = storage_config {
        config["storage_config"] = serde_json::from_str(_config).unwrap();
    }
    config.to_string()
}

pub(crate) fn build_wallet_credentials(
    key: &str,
    storage_credentials: Option<&str>,
    key_derivation_method: &str,
    rekey: Option<&str>,
    rekey_derivation_method: Option<&str>,
) -> VcxResult<String> {
    serde_json::to_string(&WalletCredentials {
        key: key.into(),
        rekey: rekey.map(|s| s.into()),
        storage_credentials: storage_credentials.map(|val| serde_json::from_str(val).unwrap()),
        key_derivation_method: key_derivation_method.into(),
        rekey_derivation_method: rekey_derivation_method.map(|s| s.into()),
    })
    .map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to serialize WalletCredentials, err: {:?}", err),
        )
    })
}

pub(crate) async fn create_indy_wallet(wallet_config: &WalletConfig) -> VcxResult<()> {
    trace!("create_wallet >>> {}", &wallet_config.wallet_name);
    let config = build_wallet_config(
        &wallet_config.wallet_name,
        wallet_config.wallet_type.as_deref(),
        wallet_config.storage_config.as_deref(),
    );
    let credentials = build_wallet_credentials(
        &wallet_config.wallet_key,
        wallet_config.storage_credentials.as_deref(),
        &wallet_config.wallet_key_derivation,
        None,
        None,
    )?;

    trace!("Credentials: {:?}", credentials);

    match wallet::create_wallet(&config, &credentials).await {
        Ok(()) => Ok(()),
        Err(err) => match err.error_code {
            ErrorCode::WalletAlreadyExistsError => {
                warn!(
                    "wallet \"{}\" already exists. skipping creation",
                    wallet_config.wallet_name
                );
                Ok(())
            }
            _ => {
                warn!(
                    "could not create wallet {}: {:?}",
                    wallet_config.wallet_name, err.message
                );
                Err(VcxError::from_msg(
                    VcxErrorKind::WalletCreate,
                    format!(
                        "could not create wallet {}: {:?}",
                        wallet_config.wallet_name, err.message
                    ),
                ))
            }
        },
    }
}

pub async fn delete_wallet(wallet_config: &WalletConfig) -> VcxResult<()> {
    trace!("delete_wallet >>> wallet_name: {}", &wallet_config.wallet_name);

    let config = build_wallet_config(
        &wallet_config.wallet_name,
        wallet_config.wallet_type.as_deref(),
        wallet_config.storage_config.as_deref(),
    );
    let credentials = build_wallet_credentials(
        &wallet_config.wallet_key,
        wallet_config.storage_credentials.as_deref(),
        &wallet_config.wallet_key_derivation,
        None,
        None,
    )?;

    wallet::delete_wallet(&config, &credentials)
        .await
        .map_err(|err| match err.error_code {
            ErrorCode::WalletAccessFailed => err.to_vcx(
                VcxErrorKind::WalletAccessFailed,
                format!(
                    "Can not open wallet \"{}\". Invalid key has been provided.",
                    &wallet_config.wallet_name
                ),
            ),
            ErrorCode::WalletNotFoundError => err.to_vcx(
                VcxErrorKind::WalletNotFound,
                format!("Wallet \"{}\" not found or unavailable", &wallet_config.wallet_name),
            ),
            error_code => err.to_vcx(VcxErrorKind::LibndyError(error_code as u32), "Indy error occurred"),
        })?;

    Ok(())
}

pub async fn import(restore_config: &RestoreWalletConfigs) -> VcxResult<()> {
    trace!(
        "import >>> wallet: {} exported_wallet_path: {}",
        restore_config.wallet_name,
        restore_config.exported_wallet_path
    );
    let new_wallet_name = restore_config.wallet_name.clone();
    let new_wallet_key = restore_config.wallet_key.clone();
    let new_wallet_kdf = restore_config
        .wallet_key_derivation
        .clone()
        .unwrap_or(settings::WALLET_KDF_DEFAULT.into());

    let new_wallet_config = build_wallet_config(&new_wallet_name, None, None);
    let new_wallet_credentials = build_wallet_credentials(&new_wallet_key, None, &new_wallet_kdf, None, None)?;
    let import_config = json!({
        "key": restore_config.backup_key,
        "path": restore_config.exported_wallet_path
    })
    .to_string();

    wallet::import_wallet(&new_wallet_config, &new_wallet_credentials, &import_config)
        .await
        .map_err(VcxError::from)
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

    vdrtools::wallet::add_wallet_record(wallet_handle, xtype, id, value, tags)
        .await
        .map_err(VcxError::from)
}

pub(crate) async fn get_wallet_record(wallet_handle: WalletHandle, xtype: &str, id: &str, options: &str) -> VcxResult<String> {
    trace!(
        "get_record >>> xtype: {}, id: {}, options: {}",
        secret!(&xtype),
        secret!(&id),
        options
    );

    if settings::indy_mocks_enabled() {
        return Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string());
    }

    vdrtools::wallet::get_wallet_record(wallet_handle, xtype, id, options)
        .await
        .map_err(VcxError::from)
}

pub(crate) async fn delete_wallet_record(wallet_handle: WalletHandle, xtype: &str, id: &str) -> VcxResult<()> {
    trace!("delete_record >>> xtype: {}, id: {}", secret!(&xtype), secret!(&id));

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    vdrtools::wallet::delete_wallet_record(wallet_handle, xtype, id)
        .await
        .map_err(VcxError::from)
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

    vdrtools::wallet::update_wallet_record_value(wallet_handle, xtype, id, value)
        .await
        .map_err(VcxError::from)
}

pub(crate) async fn add_wallet_record_tags(wallet_handle: WalletHandle, xtype: &str, id: &str, tags: &str) -> VcxResult<()> {
    trace!(
        "add_record_tags >>> xtype: {}, id: {}, tags: {:?}",
        secret!(&xtype),
        secret!(&id),
        secret!(&tags)
    );

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    vdrtools::wallet::add_wallet_record_tags(wallet_handle, xtype, id, tags)
        .await
        .map_err(VcxError::from)
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

    vdrtools::wallet::update_wallet_record_tags(wallet_handle, xtype, id, tags)
        .await
        .map_err(VcxError::from)
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

    vdrtools::wallet::delete_wallet_record_tags(wallet_handle, xtype, id, tag_names)
        .await
        .map_err(VcxError::from)
}

pub(crate) async fn open_search_wallet(
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
        return Ok(1);
    }

    vdrtools::wallet::open_wallet_search(wallet_handle, xtype, query, options)
        .await
        .map_err(VcxError::from)
}

pub(crate) async fn fetch_next_records_wallet(
    wallet_handle: WalletHandle,
    search_handle: SearchHandle,
    count: usize,
) -> VcxResult<String> {
    trace!(
        "fetch_next_records >>> search_handle: {}, count: {}",
        search_handle,
        count
    );

    if settings::indy_mocks_enabled() {
        return Ok(String::from("{}"));
    }

    vdrtools::wallet::fetch_wallet_search_next_records(wallet_handle, search_handle, count)
        .await
        .map_err(VcxError::from)
}

pub(crate) async fn close_search_wallet(search_handle: SearchHandle) -> VcxResult<()> {
    trace!("close_search >>> search_handle: {}", search_handle);

    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    vdrtools::wallet::close_wallet_search(search_handle)
        .await
        .map_err(VcxError::from)
}

// todo - can this be moved externally - move of a setup util?
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

    wallet::close_wallet(wallet_handle).await?;
    Ok(())
}

pub async fn export_wallet(wallet_handle: WalletHandle, path: &str, backup_key: &str) -> VcxResult<()> {
    trace!(
        "export >>> wallet_handle: {:?}, path: {:?}, backup_key: ****",
        wallet_handle,
        path
    );

    let export_config = json!({ "key": backup_key, "path": &path}).to_string();
    vdrtools::wallet::export_wallet(wallet_handle, &export_config)
        .await
        .map_err(VcxError::from)
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
    vdrtools::wallet::close_wallet(wallet_handle).await?;
    Ok(())
}

#[cfg(feature = "general_test")]
#[cfg(test)]
mod test {
    use crate::error::VcxErrorKind;
    use crate::indy::wallet::add_wallet_record;
    use crate::utils::devsetup::SetupLibraryWallet;

    #[tokio::test]
    async fn test_add_record() {
        let setup = SetupLibraryWallet::init().await;
        add_wallet_record(setup.wallet_handle, "record_type", "123", "Record Value", Some("{}"))
            .await
            .unwrap();
        let err = add_wallet_record(setup.wallet_handle, "record_type", "123", "Record Value", Some("{}"))
            .await
            .unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::DuplicationWalletRecord);
    }
}
