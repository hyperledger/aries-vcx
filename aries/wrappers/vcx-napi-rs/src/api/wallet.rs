#[cfg(feature = "askar_wallet")]
use libvcx_core::aries_vcx::aries_vcx_core::wallet::askar::askar_wallet_config::AskarWalletConfig;
#[cfg(feature = "vdrtools_wallet")]
use libvcx_core::aries_vcx::aries_vcx_core::wallet::indy::indy_wallet_config::IndyWalletConfig;
use libvcx_core::{
    api_vcx::api_global::{
        ledger,
        wallet::{self},
    },
    aries_vcx::aries_vcx_core::wallet::base_wallet::ManageWallet,
    errors::error::{LibvcxError, LibvcxErrorKind},
    serde_json::{self, json},
};
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

use super::napi_wallet::napi_wallet::NapiWallet;
use crate::error::to_napi_err;

#[cfg(feature = "vdrtools_wallet")]
fn parse_wallet_config(config: &str) -> napi::Result<IndyWalletConfig> {
    serde_json::from_str::<IndyWalletConfig>(config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)
}

#[cfg(feature = "askar_wallet")]
fn parse_wallet_config(config: &str) -> napi::Result<AskarWalletConfig> {
    serde_json::from_str::<AskarWalletConfig>(config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)
}

#[napi]
pub async fn wallet_open_as_main(wallet_config: String) -> napi::Result<NapiWallet> {
    let wallet_config = parse_wallet_config(&wallet_config)?;
    let wallet = wallet::wallet::open_as_main_wallet(&wallet_config)
        .await
        .map_err(to_napi_err)?;
    Ok(NapiWallet::new(wallet))
}

#[napi]
pub async fn wallet_create_main(wallet_config: String) -> napi::Result<()> {
    let wallet_config = parse_wallet_config(&wallet_config)?;
    wallet::wallet::create_main_wallet(&wallet_config)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn wallet_close_main() -> napi::Result<()> {
    wallet::wallet::close_main_wallet()
        .await
        .map_err(to_napi_err)
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
    let data = data.as_ref();
    let unpacked = wallet::wallet_unpack_message(data)
        .await
        .map_err(to_napi_err)?;
    serde_json::to_string(&unpacked).map_err(|err| napi::Error::from_reason(err.to_string()))
}

#[napi]
pub async fn create_and_store_did(seed: Option<String>) -> napi::Result<String> {
    let res = wallet::wallet_create_and_store_did(seed.as_deref())
        .await
        .map_err(to_napi_err)?;
    Ok(json!(res).to_string())
}

#[napi]
pub async fn wallet_import(config: String) -> napi::Result<()> {
    let config = serde_json::from_str(&config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    wallet::wallet::wallet_import(&config)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn wallet_export(path: String, backup_key: String) -> napi::Result<()> {
    wallet::export_main_wallet(&path, &backup_key)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn wallet_migrate(wallet_config: String) -> napi::Result<()> {
    let wallet_config = parse_wallet_config(&wallet_config)?;

    wallet::wallet::wallet_migrate(&wallet_config)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub async fn wallet_delete(wallet_config: String) -> napi::Result<()> {
    let wallet_config = parse_wallet_config(&wallet_config)?;

    wallet_config
        .delete_wallet()
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
    wallet::replace_did_keys_start(&did)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn rotate_verkey_apply(did: String, temp_vk: String) -> napi::Result<()> {
    wallet::rotate_verkey_apply(&did, &temp_vk)
        .await
        .map_err(to_napi_err)
}
