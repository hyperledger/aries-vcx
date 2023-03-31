use napi_derive::napi;

use libvcx_core::api_vcx::api_handle::disclosed_proof;

use crate::error::to_napi_err;

#[napi]
fn disclosed_proof_create_with_request(source_id: String, proof_req: String) -> napi::Result<u32> {
    disclosed_proof::create_with_proof_request(&source_id, &proof_req).map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_release(handle: u32) -> napi::Result<()> {
    disclosed_proof::release(handle).map_err(to_napi_err)
}

#[napi]
async fn disclosed_proof_send_proof(handle: u32, handle_connection: u32) -> napi::Result<()> {
    disclosed_proof::send_proof(handle, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn disclosed_proof_reject_proof(handle: u32, handle_connection: u32) -> napi::Result<()> {
    disclosed_proof::reject_proof(handle, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_get_proof_msg(handle: u32) -> napi::Result<String> {
    disclosed_proof::get_presentation_msg(handle).map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_serialize(handle: u32) -> napi::Result<String> {
    disclosed_proof::to_string(handle).map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_deserialize(data: String) -> napi::Result<u32> {
    disclosed_proof::from_string(&data).map_err(to_napi_err)
}

#[napi]
async fn v2_disclosed_proof_update_state(handle: u32, connection_handle: u32) -> napi::Result<u32> {
    disclosed_proof::update_state(handle, None, connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn v2_disclosed_proof_update_state_with_message(
    handle: u32,
    message: String,
    connection_handle: u32,
) -> napi::Result<u32> {
    disclosed_proof::update_state(handle, Some(&message), connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_get_state(handle: u32) -> napi::Result<u32> {
    disclosed_proof::get_state(handle).map_err(to_napi_err)
}

// todo: move to mediated connection
#[napi]
async fn disclosed_proof_get_requests(handle_connection: u32) -> napi::Result<String> {
    disclosed_proof::get_proof_request_messages(handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn disclosed_proof_retrieve_credentials(handle: u32) -> napi::Result<String> {
    disclosed_proof::retrieve_credentials(handle).await.map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_get_proof_request_attachment(handle: u32) -> napi::Result<String> {
    disclosed_proof::get_proof_request_attachment(handle).map_err(to_napi_err)
}

#[napi]
async fn disclosed_proof_generate_proof(
    handle: u32,
    credentials: String,
    self_attested_attrs: String,
) -> napi::Result<()> {
    disclosed_proof::generate_proof(handle, &credentials, &self_attested_attrs)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn disclosed_proof_decline_presentation_request(
    handle: u32,
    connection_handle: u32,
    reason: Option<String>,
    proposal: Option<String>,
) -> napi::Result<()> {
    disclosed_proof::decline_presentation_request(handle, connection_handle, reason.as_deref(), proposal.as_deref())
        .await
        .map_err(to_napi_err)
}

#[napi]
fn disclosed_proof_get_thread_id(handle: u32) -> napi::Result<String> {
    disclosed_proof::get_thread_id(handle).map_err(to_napi_err)
}
