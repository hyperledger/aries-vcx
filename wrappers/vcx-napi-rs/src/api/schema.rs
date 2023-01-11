use napi_derive::napi;

use crate::error::to_napi_err;
use vcx::api_vcx::api_handle::schema;

#[napi]
fn schema_get_attributes(_source_id: String, _schema_id: String) -> napi::Result<()> {
    unimplemented!("Not implemented in napi wrapper yet")
}

#[napi]
fn schema_prepare_for_endorser() -> napi::Result<()> {
    unimplemented!("Not implemented in napi wrapper yet")
}

#[napi]
async fn schema_create(source_id: String, name: String, version: String, data: String) -> napi::Result<u32> {
    schema::create_and_publish_schema(&source_id, name, version, data)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn schema_get_schema_id(handle_schema: u32) -> napi::Result<String> {
    schema::get_schema_id(handle_schema).map_err(to_napi_err)
}

#[napi]
fn schema_deserialize(serialized: String) -> napi::Result<u32> {
    schema::from_string(&serialized).map_err(to_napi_err)
}

#[napi]
fn schema_serialize(handle_schema: u32) -> napi::Result<String> {
    schema::to_string(handle_schema).map_err(to_napi_err)
}

#[napi]
fn schema_release(handle_schema: u32) -> napi::Result<()> {
    schema::release(handle_schema).map_err(to_napi_err)
}

#[napi]
async fn schema_update_state(handle_schema: u32) -> napi::Result<u32> {
    schema::update_state(handle_schema).await.map_err(to_napi_err)
}

#[napi]
fn schema_get_state(handle_schema: u32) -> napi::Result<u32> {
    schema::get_state(handle_schema).map_err(to_napi_err)
}
