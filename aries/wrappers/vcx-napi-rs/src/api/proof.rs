use libvcx_core::api_vcx::api_handle::proof;
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
async fn proof_create(
    source_id: String,
    requested_attrs: String,
    requested_predicates: String,
    revocation_details: String,
    name: String,
) -> napi::Result<u32> {
    proof::create_proof(
        source_id,
        requested_attrs,
        requested_predicates,
        revocation_details,
        name,
    )
    .await
    .map_err(to_napi_err)
}

#[napi]
fn proof_get_presentation_msg(handle: u32) -> napi::Result<String> {
    proof::get_presentation_msg(handle).map_err(to_napi_err)
}

#[napi]
fn proof_get_presentation_request_attachment(handle: u32) -> napi::Result<String> {
    proof::get_presentation_request_attachment(handle).map_err(to_napi_err)
}

#[napi]
fn proof_get_presentation_attachment(handle: u32) -> napi::Result<String> {
    proof::get_presentation_attachment(handle).map_err(to_napi_err)
}

#[napi]
fn proof_release(handle: u32) -> napi::Result<()> {
    proof::release(handle).map_err(to_napi_err)
}

#[napi]
async fn proof_send_request(handle_proof: u32, handle_connection: u32) -> napi::Result<()> {
    proof::send_proof_request(handle_proof, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn proof_send_request_nonmediated(
    handle_proof: u32,
    handle_connection: u32,
) -> napi::Result<()> {
    proof::send_proof_request_nonmediated(handle_proof, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn proof_get_request_msg(handle: u32) -> napi::Result<String> {
    proof::get_presentation_request_msg(handle).map_err(to_napi_err)
}

#[napi]
fn proof_serialize(handle: u32) -> napi::Result<String> {
    proof::to_string(handle).map_err(to_napi_err)
}

#[napi]
fn proof_deserialize(data: String) -> napi::Result<u32> {
    proof::from_string(&data).map_err(to_napi_err)
}

// todo: fix argument order
#[napi]
async fn v2_proof_update_state(handle_proof: u32, connection_handle: u32) -> napi::Result<u32> {
    proof::update_state(handle_proof, None, connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn v2_proof_update_state_with_message(
    handle_proof: u32,
    message: String,
    connection_handle: u32,
) -> napi::Result<u32> {
    proof::update_state(handle_proof, Some(&message), connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn proof_update_state_with_message_nonmediated(
    handle_proof: u32,
    connection_handle: u32,
    message: String,
) -> napi::Result<u32> {
    proof::update_state_nonmediated(handle_proof, connection_handle, &message)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn proof_get_state(handle: u32) -> napi::Result<u32> {
    proof::get_state(handle).map_err(to_napi_err)
}

#[napi]
fn proof_get_verification_status(handle: u32) -> napi::Result<u32> {
    proof::get_verification_status(handle)
        .map_err(to_napi_err)
        .map(|status| status.code())
}

#[napi]
fn proof_get_thread_id(handle: u32) -> napi::Result<String> {
    proof::get_thread_id(handle).map_err(to_napi_err)
}

#[napi]
fn mark_presentation_request_msg_sent(handle: u32) -> napi::Result<()> {
    proof::mark_presentation_request_msg_sent(handle).map_err(to_napi_err)
}
