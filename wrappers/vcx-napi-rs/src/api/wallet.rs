use libvcx_core::api_vcx::api_global::settings::settings_init_issuer_config;
use libvcx_core::api_vcx::api_global::{ledger, wallet};
use libvcx_core::aries_vcx::aries_vcx_core::indy::wallet::{IssuerConfig, RestoreWalletConfigs, WalletConfig};
use libvcx_core::errors::error::{LibvcxError, LibvcxErrorKind};
use libvcx_core::serde_json;
use libvcx_core::serde_json::json;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
pub async fn wallet_open_as_main(wallet_config: String) -> napi::Result<i32> {
    let wallet_config = serde_json::from_str::<WalletConfig>(&wallet_config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    let handle = wallet::open_as_main_wallet(&wallet_config).await.map_err(to_napi_err)?;
    Ok(handle.0)
}

#[napi]
pub async fn wallet_create_main(wallet_config: String) -> napi::Result<()> {
    let wallet_config = serde_json::from_str::<WalletConfig>(&wallet_config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    wallet::create_main_wallet(&wallet_config).await.map_err(to_napi_err)
}

#[napi]
pub async fn wallet_close_main() -> napi::Result<()> {
    wallet::close_main_wallet().await.map_err(to_napi_err)
}

#[napi]
pub async fn vcx_init_issuer_config(config: String) -> napi::Result<()> {
    let config = serde_json::from_str::<IssuerConfig>(&config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    settings_init_issuer_config(&config).map_err(to_napi_err)
}

#[napi]
pub async fn configure_issuer_wallet(enterprise_seed: String) -> napi::Result<String> {
    let res = wallet::wallet_configure_issuer(&enterprise_seed)
        .await
        .map_err(to_napi_err)?;
    Ok(json!(res).to_string())
}

#[napi]
pub async fn unpack(data: Buffer) -> napi::Result<String> {
    wallet::wallet_unpack_message_to_string(&data.to_vec())
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn create_pairwise_info() -> napi::Result<String> {
    let res = wallet::wallet_create_pairwise_did().await.map_err(to_napi_err)?;
    Ok(json!(res).to_string())
}

#[napi]
pub async fn wallet_import(config: String) -> napi::Result<()> {
    let config = serde_json::from_str::<RestoreWalletConfigs>(&config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    wallet::wallet_import(&config).await.map_err(to_napi_err)
}

#[napi]
pub async fn wallet_export(path: String, backup_key: String) -> napi::Result<()> {
    wallet::export_main_wallet(&path, &backup_key)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn wallet_migrate(wallet_config: String) -> napi::Result<()> {
    let wallet_config = serde_json::from_str(&wallet_config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;

    wallet::wallet_migrate(&wallet_config)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub async fn get_verkey_from_wallet(did: String) -> napi::Result<String> {
    wallet::key_for_local_did(&did).await.map_err(to_napi_err)
}

#[napi]
pub async fn rotate_verkey(did: String) -> napi::Result<()> {
    ledger::rotate_verkey(&did).await.map_err(to_napi_err)
}

#[napi]
pub async fn rotate_verkey_start(did: String) -> napi::Result<String> {
    wallet::replace_did_keys_start(&did).await.map_err(to_napi_err)
}

#[napi]
pub async fn rotate_verkey_apply(did: String, temp_vk: String) -> napi::Result<()> {
    wallet::rotate_verkey_apply(&did, &temp_vk).await.map_err(to_napi_err)
}
