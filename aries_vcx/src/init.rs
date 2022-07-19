use indy;
use indy::ErrorCode;
use indy_sys::WalletHandle;

use crate::error::{VcxErrorExt, VcxErrorKind, VcxResult};
use crate::libindy::utils::pool::{create_pool_ledger_config, open_pool_ledger};
use crate::libindy::utils::wallet::{build_wallet_config, build_wallet_credentials, IssuerConfig, set_wallet_handle, WalletConfig};
use crate::settings;
use crate::utils::provision::AgencyClientConfig;

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct PoolConfig {
    pub genesis_path: String,
    pub pool_name: Option<String>,
    pub pool_config: Option<String>,
}

pub fn enable_vcx_mocks() -> VcxResult<()> {
    info!("enable_vcx_mocks >>>");
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
    Ok(())
}

pub fn enable_agency_mocks() -> VcxResult<()> {
    info!("enable_agency_mocks >>>");
    settings::get_agency_client_mut()?.enable_test_mode();
    Ok(())
}

pub fn create_agency_client_for_main_wallet(config: &AgencyClientConfig) -> VcxResult<()> {
    let config = serde_json::to_string(config).unwrap();
    // todo: remove unwrap
    info!("init_agency_client >>> config = {}", config);
    settings::get_agency_client_mut()?.process_config_string(&config, false)?;
    Ok(())
}

pub fn init_issuer_config(config: &IssuerConfig) -> VcxResult<()> {
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &config.institution_did);
    Ok(())
}

pub async fn open_main_pool(config: &PoolConfig) -> VcxResult<()> {
    let pool_name = config.pool_name.clone().unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
    trace!("open_pool >>> pool_name: {}, path: {}, pool_config: {:?}", pool_name, config.genesis_path, config.pool_config);

    create_pool_ledger_config(&pool_name, &config.genesis_path)
        .await
        .map_err(|err| err.extend("Can not create Pool Ledger Config"))?;

    debug!("open_pool ::: Pool Config Created Successfully");

    open_pool_ledger(&pool_name, config.pool_config.as_deref())
        .await
        .map_err(|err| err.extend("Can not open Pool Ledger"))?;

    info!("open_pool ::: Pool Opened Successfully");

    Ok(())
}

pub async fn open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
    trace!("open_as_main_wallet >>> {}", &wallet_config.wallet_name);
    let config = build_wallet_config(&wallet_config.wallet_name, wallet_config.wallet_type.as_deref(), wallet_config.storage_config.as_deref());
    let credentials = build_wallet_credentials(&wallet_config.wallet_key, wallet_config.storage_credentials.as_deref(), &wallet_config.wallet_key_derivation, wallet_config.rekey.as_deref(), wallet_config.rekey_derivation_method.as_deref())?;

    let handle = indy::wallet::open_wallet(&config, &credentials)
        .await
        .map_err(|err|
            match err.error_code.clone() {
                ErrorCode::WalletAlreadyOpenedError => {
                    err.to_vcx(VcxErrorKind::WalletAlreadyOpen,
                               format!("Wallet \"{}\" already opened.", wallet_config.wallet_name))
                }
                ErrorCode::WalletAccessFailed => {
                    err.to_vcx(VcxErrorKind::WalletAccessFailed,
                               format!("Can not open wallet \"{}\". Invalid key has been provided.", wallet_config.wallet_name))
                }
                ErrorCode::WalletNotFoundError => {
                    err.to_vcx(VcxErrorKind::WalletNotFound,
                               format!("Wallet \"{}\" not found or unavailable", wallet_config.wallet_name))
                }
                error_code => {
                    err.to_vcx(VcxErrorKind::LibndyError(error_code as u32), "Indy error occurred")
                }
            })?;

    set_wallet_handle(handle);

    Ok(handle)
}
