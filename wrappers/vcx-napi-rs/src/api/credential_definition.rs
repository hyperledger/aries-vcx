use napi_derive::napi;

use crate::error::to_napi_err;
use vcx::api_vcx::api_handle::credential_def;

#[napi]
async fn credentialdef_create_v2_(
    source_id: String,
    schema_id: String,
    tag: String,
    support_revocation: bool,
) -> napi::Result<u32> {
    credential_def::create(source_id, schema_id, tag, support_revocation)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn credentialdef_publish(handle: u32) -> napi::Result<()> {
    credential_def::publish(handle).await.map_err(to_napi_err)
}

#[napi]
fn credentialdef_deserialize(serialized: String) -> napi::Result<u32> {
    credential_def::from_string(&serialized).map_err(to_napi_err)
}

#[napi]
fn credentialdef_release(handle: u32) -> napi::Result<()> {
    credential_def::release(handle).map_err(to_napi_err)
}

#[napi]
fn credentialdef_serialize(handle: u32) -> napi::Result<String> {
    credential_def::to_string(handle).map_err(to_napi_err)
}

#[napi]
fn credentialdef_get_cred_def_id(handle: u32) -> napi::Result<String> {
    credential_def::get_cred_def_id(handle).map_err(to_napi_err)
}

#[napi]
async fn credentialdef_update_state(handle: u32) -> napi::Result<u32> {
    credential_def::update_state(handle).await.map_err(to_napi_err)
}

#[napi]
fn credentialdef_get_state(handle: u32) -> napi::Result<u32> {
    credential_def::get_state(handle).map_err(to_napi_err)
}
