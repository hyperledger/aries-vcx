use libvcx_core::{
    api_vcx::api_handle::{revocation_registry, revocation_registry::RevocationRegistryConfig},
    errors::error::{LibvcxError, LibvcxErrorKind},
    serde_json,
};
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
async fn revocation_registry_create(config: String) -> napi::Result<u32> {
    let config = serde_json::from_str::<RevocationRegistryConfig>(&config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidJson,
                format!("Serialization error: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    revocation_registry::create(config)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn revocation_registry_publish(handle: u32, tails_url: String) -> napi::Result<u32> {
    revocation_registry::publish(handle, &tails_url)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn revocation_registry_publish_revocations(handle: u32) -> napi::Result<()> {
    revocation_registry::publish_revocations(handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn revocation_registry_get_rev_reg_id(handle: u32) -> napi::Result<String> {
    revocation_registry::get_rev_reg_id(handle).map_err(to_napi_err)
}

#[napi]
fn revocation_registry_get_tails_hash(handle: u32) -> napi::Result<String> {
    revocation_registry::get_tails_hash(handle).map_err(to_napi_err)
}

#[napi]
fn revocation_registry_serialize(handle: u32) -> napi::Result<String> {
    revocation_registry::to_string(handle).map_err(to_napi_err)
}

#[napi]
fn revocation_registry_deserialize(data: String) -> napi::Result<u32> {
    revocation_registry::from_string(&data).map_err(to_napi_err)
}

#[napi]
fn revocation_registry_release(handle: u32) -> napi::Result<()> {
    revocation_registry::release(handle).map_err(to_napi_err)
}
