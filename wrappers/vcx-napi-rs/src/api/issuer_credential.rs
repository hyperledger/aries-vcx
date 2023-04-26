use crate::error::to_napi_err;
use libvcx_core::api_vcx::api_handle::issuer_credential;
use libvcx_core::aries_vcx::messages::AriesMessage;
use libvcx_core::serde_json::json;
use napi_derive::napi;

#[napi]
fn issuer_credential_deserialize(credential_data: String) -> napi::Result<u32> {
    issuer_credential::from_string(&credential_data).map_err(to_napi_err)
}

#[napi]
fn issuer_credential_serialize(handle_credential: u32) -> napi::Result<String> {
    issuer_credential::to_string(handle_credential).map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_update_state_v2(handle_credential: u32, connection_handle: u32) -> napi::Result<u32> {
    issuer_credential::update_state(handle_credential, None, connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_update_state_with_message_v2(
    handle_credential: u32,
    connection_handle: u32,
    message: String,
) -> napi::Result<u32> {
    issuer_credential::update_state(handle_credential, Some(&message), connection_handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_update_state_with_message_nonmediated(
    handle_credential: u32,
    connection_handle: u32,
    message: String,
) -> napi::Result<u32> {
    issuer_credential::update_state_with_message_nonmediated(handle_credential, connection_handle, &message)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn issuer_credential_get_state(handle_credential: u32) -> napi::Result<u32> {
    issuer_credential::get_state(handle_credential).map_err(to_napi_err)
}

#[napi]
fn issuer_credential_get_rev_reg_id(handle_credential: u32) -> napi::Result<String> {
    issuer_credential::get_rev_reg_id(handle_credential).map_err(to_napi_err)
}

#[napi]
fn issuer_credential_create(source_id: String) -> napi::Result<u32> {
    issuer_credential::issuer_credential_create(source_id).map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_revoke_local(handle_credential: u32) -> napi::Result<()> {
    issuer_credential::revoke_credential_local(handle_credential)
        .await
        .map_err(to_napi_err)
}

#[napi]
fn issuer_credential_is_revokable(handle_credential: u32) -> napi::Result<bool> {
    issuer_credential::is_revokable(handle_credential).map_err(to_napi_err)
}

#[napi]
fn issuer_credential_get_revocation_id(handle_credential: u32) -> napi::Result<String> {
    issuer_credential::get_revocation_id(handle_credential).map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_send_credential(handle_credential: u32, handle_connection: u32) -> napi::Result<u32> {
    issuer_credential::send_credential(handle_credential, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_send_credential_nonmediated(
    handle_credential: u32,
    handle_connection: u32,
) -> napi::Result<u32> {
    issuer_credential::send_credential_nonmediated(handle_credential, handle_connection)
        .await
        .map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_send_offer_v2(handle_credential: u32, handle_connection: u32) -> napi::Result<()> {
    issuer_credential::send_credential_offer_v2(handle_credential, handle_connection)
        .await
        .map_err(to_napi_err)?;
    Ok(())
}

#[napi]
async fn issuer_credential_send_offer_nonmediated(handle_credential: u32, handle_connection: u32) -> napi::Result<()> {
    issuer_credential::send_credential_offer_nonmediated(handle_credential, handle_connection)
        .await
        .map_err(to_napi_err)?;
    Ok(())
}

#[napi]
fn issuer_credential_mark_offer_msg_sent(handle_credential: u32) -> napi::Result<()> {
    issuer_credential::mark_credential_offer_msg_sent(handle_credential).map_err(to_napi_err)
}

#[napi]
async fn issuer_credential_build_offer_msg_v2(
    credential_handle: u32,
    cred_def_handle: u32,
    rev_reg_handle: u32,
    credential_json: String,
    comment: Option<String>,
) -> napi::Result<()> {
    issuer_credential::build_credential_offer_msg_v2(
        credential_handle,
        cred_def_handle,
        rev_reg_handle,
        &credential_json,
        comment.as_deref(),
    )
    .await
    .map_err(to_napi_err)
}

#[napi]
fn issuer_credential_get_offer_msg(credential_handle: u32) -> napi::Result<String> {
    let res: AriesMessage = issuer_credential::get_credential_offer_msg(credential_handle).map_err(to_napi_err)?;
    Ok(json!(res).to_string())
}

#[napi]
fn issuer_credential_release(credential_handle: u32) -> napi::Result<()> {
    issuer_credential::release(credential_handle).map_err(to_napi_err)
}

#[napi]
fn issuer_credential_get_thread_id(credential_handle: u32) -> napi::Result<String> {
    issuer_credential::get_thread_id(credential_handle).map_err(to_napi_err)
}
