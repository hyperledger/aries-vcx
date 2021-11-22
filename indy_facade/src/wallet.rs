use indy::future::Future;
use indyrs::{ErrorCode, wallet};
use crate::error::{IndyFacadeError, IndyFacadeErrorKind, IndyFacadeResult, VcxErrorExt};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletCredentials {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_credentials: Option<serde_json::Value>,
    pub key_derivation_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_derivation_method: Option<String>,
}

pub fn build_wallet_config(wallet_name: &str, wallet_type: Option<&str>, storage_config: Option<&str>) -> String {
    let mut config = json!({
        "id": wallet_name,
        "storage_type": wallet_type
    });
    if let Some(_config) = storage_config { config["storage_config"] = serde_json::from_str(_config).unwrap(); }
    config.to_string()
}

pub fn delete_wallet(wallet_config: &WalletConfig) -> IndyFacadeResult<()> {
    trace!("delete_wallet >>> wallet_name: {}", &wallet_config.wallet_name);

    let config = build_wallet_config(&wallet_config.wallet_name, wallet_config.wallet_type.as_ref().map(String::as_str), wallet_config.storage_config.as_deref());
    let credentials = build_wallet_credentials(&wallet_config.wallet_key, wallet_config.storage_credentials.as_deref(), &wallet_config.wallet_key_derivation, None, None)?;

    wallet::delete_wallet(&config, &credentials)
        .wait()
        .map_err(|err|
            match err.error_code.clone() {
                ErrorCode::WalletAccessFailed => {
                    err.to_indy_facade_err(IndyFacadeErrorKind::WalletAccessFailed,
                                           format!("Can not open wallet \"{}\". Invalid key has been provided.", &wallet_config.wallet_name))
                }
                ErrorCode::WalletNotFoundError => {
                    err.to_indy_facade_err(IndyFacadeErrorKind::WalletNotFound,
                                           format!("Wallet \"{}\" not found or unavailable", &wallet_config.wallet_name))
                }
                error_code => {
                    err.to_indy_facade_err(IndyFacadeErrorKind::LibndyError(error_code as u32), "Indy error occurred")
                }
            })?;

    Ok(())
}

pub fn build_wallet_credentials(key: &str, storage_credentials: Option<&str>, key_derivation_method: &str, rekey: Option<&str>, rekey_derivation_method: Option<&str>) -> IndyFacadeResult<String> {
    serde_json::to_string(&WalletCredentials {
        key: key.into(),
        rekey: rekey.map(|s| s.into()),
        storage_credentials: storage_credentials.map(|val| serde_json::from_str(val).unwrap()),
        key_derivation_method: key_derivation_method.into(),
        rekey_derivation_method: rekey_derivation_method.map(|s| s.into()),
    }).map_err(|err| IndyFacadeError::from_msg(IndyFacadeErrorKind::SerializationError, format!("Failed to serialize WalletCredentials, err: {:?}", err)))
}

pub fn create_indy_wallet(wallet_config: &WalletConfig) -> IndyFacadeResult<()> {
    trace!("create_wallet >>> {}", &wallet_config.wallet_name);
    let config = build_wallet_config(
        &wallet_config.wallet_name,
        wallet_config.wallet_type.as_deref(),
        wallet_config.storage_config.as_deref());
    let credentials = build_wallet_credentials(
        &wallet_config.wallet_key,
        wallet_config.storage_credentials.as_deref(),
        &wallet_config.wallet_key_derivation,
        None,
        None,
    )?;

    trace!("Credentials: {:?}", credentials);

    match wallet::create_wallet(&config, &credentials)
        .wait() {
        Ok(()) => Ok(()),
        Err(err) => {
            match err.error_code.clone() {
                ErrorCode::WalletAlreadyExistsError => {
                    warn!("wallet \"{}\" already exists. skipping creation", wallet_config.wallet_name);
                    Ok(())
                }
                _ => {
                    warn!("could not create wallet {}: {:?}", wallet_config.wallet_name, err.message);
                    Err(IndyFacadeError::from_msg(IndyFacadeErrorKind::WalletCreate, format!("could not create wallet {}: {:?}", wallet_config.wallet_name, err.message)))
                }
            }
        }
    }
}
