use crate::error::{VcxErrorExt, VcxErrorKind, VcxResult};
use crate::indy::{ErrorCode, WalletHandle};
use crate::libindy::utils::wallet::{build_wallet_config, build_wallet_credentials, WalletConfig};

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

    let handle = indy::wallet::open_wallet(&config, &credentials)
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
