use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use libvcx_core::api_vcx::api_handle::mediated_connection;
use libvcx_core::api_vcx::api_handle::mediated_connection::parse_status_codes;

use libvcx_core::aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
use libvcx_core::errors::error::{LibvcxError, LibvcxErrorKind};
use libvcx_core::serde_json;

use crate::error::to_napi_err;

#[napi]
pub fn mediated_connection_generate_public_invite(public_did: String, label: String) -> napi::Result<String> {
    trace!(
        "mediated_connection_generate_public_invite >>> public_did: {:?}, label: {:?}",
        public_did,
        label
    );
    mediated_connection::generate_public_invitation(&public_did, &label).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_get_pw_did(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_get_pw_did >>> handle: {:?}", handle);
    mediated_connection::get_pw_did(handle).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_get_their_pw_did(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_get_their_pw_did >>> handle: {:?}", handle);
    mediated_connection::get_their_pw_did(handle).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_get_thread_id(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_get_thread_id >>> handle: {:?}", handle);
    mediated_connection::get_thread_id(handle).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_get_state(handle: u32) -> u32 {
    trace!("mediated_connection_get_state >>> handle: {:?}", handle);
    mediated_connection::get_state(handle)
}

#[napi]
pub fn mediated_connection_get_source_id(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_get_source_id >>> handle: {:?}", handle);
    mediated_connection::get_source_id(handle).map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_create(source_id: String) -> napi::Result<u32> {
    trace!("mediated_connection_create >>> source_id: {:?}", source_id);
    mediated_connection::create_connection(&source_id)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_create_with_invite(source_id: String, details: String) -> napi::Result<u32> {
    trace!(
        "mediated_connection_create_with_invite >>> source_id: {:?}, details: {:?}",
        source_id,
        details
    );
    mediated_connection::create_connection_with_invite(&source_id, &details)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_send_message(handle: u32, msg: String) -> napi::Result<()> {
    trace!(
        "mediated_connection_send_message >>> handle: {:?}, msg: {:?}",
        handle,
        msg
    );
    mediated_connection::send_generic_message(handle, &msg)
        .await
        .map(|_res| ())
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_create_with_connection_request_v2(
    request: String,
    pw_info: String,
) -> napi::Result<u32> {
    trace!(
        "mediated_connection_create_with_connection_request_v2 >>> request: {:?}, pw_info: {:?}",
        request,
        pw_info
    );
    let pw_info: PairwiseInfo = serde_json::from_str(&pw_info)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidJson,
                format!("Cannot deserialize pw info: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    mediated_connection::create_with_request_v2(&request, pw_info)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_send_handshake_reuse(handle: u32, oob_msg: String) -> napi::Result<()> {
    trace!(
        "mediated_connection_send_handshake_reuse >>> handle: {:?}, oob_msg: {:?}",
        handle,
        oob_msg
    );
    mediated_connection::send_handshake_reuse(handle, &oob_msg)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_update_state_with_message(handle: u32, message: String) -> napi::Result<u32> {
    trace!(
        "mediated_connection_update_state_with_message >>> handle: {:?}, message: {:?}",
        handle,
        message
    );
    mediated_connection::update_state_with_message(handle, &message)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_handle_message(handle: u32, message: String) -> napi::Result<()> {
    trace!(
        "mediated_connection_handle_message >>> handle: {:?}, message: {:?}",
        handle,
        message
    );
    mediated_connection::handle_message(handle, &message)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_update_state(handle: u32) -> napi::Result<u32> {
    trace!("mediated_connection_update_state >>> handle: {:?}", handle);
    mediated_connection::update_state(handle).await.map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_delete_connection(handle: u32) -> napi::Result<()> {
    trace!("mediated_connection_delete_connection >>> handle: {:?}", handle);
    mediated_connection::delete_connection(handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_connect(handle: u32) -> napi::Result<()> {
    trace!("mediated_connection_connect >>> handle: {:?}", handle);
    mediated_connection::connect(handle).await.map_err(to_napi_err)?;
    Ok(())
}

#[napi]
pub fn mediated_connection_serialize(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_serialize >>> handle: {:?}", handle);
    mediated_connection::to_string(handle).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_deserialize(connection_data: String) -> napi::Result<u32> {
    trace!(
        "mediated_connection_deserialize >>> connection_data: {:?}",
        connection_data
    );
    mediated_connection::from_string(&connection_data).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_release(handle: u32) -> napi::Result<()> {
    trace!("mediated_connection_release >>> handle: {:?}", handle);
    mediated_connection::release(handle).map_err(to_napi_err)
}

#[napi]
pub fn mediated_connection_invite_details(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_invite_details >>> handle: {:?}", handle);
    mediated_connection::get_invite_details(handle).map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_send_ping(handle: u32, comment: Option<String>) -> napi::Result<()> {
    trace!(
        "mediated_connection_send_ping >>> handle: {:?}, comment: {:?}",
        handle,
        comment
    );
    mediated_connection::send_ping(handle, comment.as_deref())
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_send_discovery_features(
    handle: u32,
    query: Option<String>,
    comment: Option<String>,
) -> napi::Result<()> {
    trace!(
        "mediated_connection_send_discovery_features >>> handle: {:?}, query: {:?}, comment: {:?}",
        handle,
        query,
        comment
    );
    mediated_connection::send_discovery_features(handle, query.as_deref(), comment.as_deref())
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_info(handle: u32) -> napi::Result<String> {
    trace!("mediated_connection_info >>> handle: {:?}", handle);
    mediated_connection::get_connection_info(handle)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_messages_download(
    conn_handles: Vec<u32>,
    status_codes: Option<String>,
    uids: Option<String>,
) -> napi::Result<String> {
    trace!(
        "mediated_connection_messages_download >>> conn_handles: {:?}, status_codes: {:?}, uids: {:?}",
        conn_handles,
        status_codes,
        uids
    );
    let status_codes = if let Some(status_codes) = status_codes {
        let v: Vec<&str> = status_codes.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v)
    } else {
        None
    };

    let status_codes = parse_status_codes(status_codes).map_err(to_napi_err)?;

    let uids = if let Some(uids) = uids {
        let v: Vec<&str> = uids.split(',').collect();
        let v = v.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        Some(v)
    } else {
        None
    };

    mediated_connection::download_messages(conn_handles, status_codes, uids)
        .await
        .map(|msgs| serde_json::json!(msgs).to_string())
        .map_err(to_napi_err)
}

#[napi]
pub async fn mediated_connection_sign_data(handle: u32, data: Buffer) -> napi::Result<Buffer> {
    trace!("mediated_connection_sign_data >>> handle: {:?}", handle);
    let res = mediated_connection::sign_data(handle, &data.to_vec())
        .await
        .map_err(to_napi_err)?;
    Ok(Buffer::from(res))
}

#[napi]
pub async fn mediated_connection_verify_signature(handle: u32, data: Buffer, signature: Buffer) -> napi::Result<bool> {
    trace!("mediated_connection_verify_signature >>> handle: {:?}", handle);
    mediated_connection::verify_signature(handle, &data.to_vec(), &signature.to_vec())
        .await
        .map_err(to_napi_err)
}
