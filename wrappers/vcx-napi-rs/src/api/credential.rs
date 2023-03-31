use napi_derive::napi;

use libvcx_core::api_vcx::api_handle::credential;

use crate::error::to_napi_err;

#[napi]
fn credential_create_with_offer(source_id: String, offer: String) -> napi::Result<u32> {
    credential::credential_create_with_offer(&source_id, &offer).map_err(to_napi_err)
}

#[napi]
fn credential_release(handle: u32) -> napi::Result<()> {
    credential::release(handle).map_err(to_napi_err)
}

#[napi]
async fn credential_send_request(handle: u32, handle_connection: u32) -> napi::Result<()> {
    credential::send_credential_request(handle, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn credential_decline_offer(handle: u32, handle_connection: u32, comment: Option<String>) -> napi::Result<()> {
    credential::decline_offer(handle, handle_connection, comment.as_deref())
        .await
        .map_err(to_napi_err)
}

#[napi]
fn credential_serialize(handle: u32) -> napi::Result<String> {
    credential::to_string(handle).map_err(to_napi_err)
}

#[napi]
fn credential_deserialize(data: String) -> napi::Result<u32> {
    credential::from_string(&data).map_err(to_napi_err)
}

// todo: flip order of arguments
#[napi]
async fn v2_credential_update_state_with_message(
    handle_credential: u32,
    message: Option<String>,
    connection_handle: u32,
) -> napi::Result<u32> {
    credential::update_state(handle_credential, message.as_deref(), connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn v2_credential_update_state(handle_credential: u32, connection_handle: u32) -> napi::Result<u32> {
    credential::update_state(handle_credential, None, connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn credential_get_state(handle: u32) -> napi::Result<u32> {
    credential::get_state(handle).map_err(to_napi_err)
}

// todo: move this to mediated_connection
#[napi]
async fn credential_get_offers(handle_connection: u32) -> napi::Result<String> {
    credential::get_credential_offer_messages_with_conn_handle(handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn credential_get_attributes(handle: u32) -> napi::Result<String> {
    credential::get_attributes(handle).map_err(to_napi_err)
}

#[napi]
fn credential_get_attachment(handle: u32) -> napi::Result<String> {
    credential::get_attachment(handle).map_err(to_napi_err)
}

#[napi]
fn credential_get_tails_location(handle: u32) -> napi::Result<String> {
    credential::get_tails_location(handle).map_err(to_napi_err)
}

#[napi]
fn credential_get_tails_hash(handle: u32) -> napi::Result<String> {
    credential::get_tails_hash(handle).map_err(to_napi_err)
}

#[napi]
fn credential_get_rev_reg_id(handle: u32) -> napi::Result<String> {
    credential::get_rev_reg_id(handle).map_err(to_napi_err)
}

#[napi]
fn credential_get_thread_id(handle: u32) -> napi::Result<String> {
    credential::get_thread_id(handle).map_err(to_napi_err)
}
