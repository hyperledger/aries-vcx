use time;
use base64;
use vdrtools::crypto;
use vdrtools_sys::WalletHandle;

use crate::error::prelude::*;
use crate::global::settings;
use crate::messages::connection::response::{Response, SignedResponse, ConnectionSignature, ConnectionData};

pub async fn sign(wallet_handle: WalletHandle, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    crypto::sign(wallet_handle, my_vk, msg).await.map_err(VcxError::from)
}

async fn get_signature_data(wallet_handle: WalletHandle, data: String, key: &str) -> VcxResult<(Vec<u8>, Vec<u8>)> {
    let now: u64 = time::get_time().sec as u64;
    let mut sig_data = now.to_be_bytes().to_vec();
    sig_data.extend(data.as_bytes());
    
    let signature = sign(wallet_handle, key, &sig_data).await?;
    
    Ok((signature, sig_data))
}

pub async fn sign_connection_response(wallet_handle: WalletHandle, key: &str, response: Response) -> VcxResult<SignedResponse> {
    let connection_data = response.get_connection_data();
    let (signature, sig_data) = get_signature_data(wallet_handle, connection_data, key).await?;
    
    let sig_data = base64::encode_config(&sig_data, base64::URL_SAFE);
    let signature = base64::encode_config(&signature, base64::URL_SAFE);

    let connection_sig = ConnectionSignature {
        signature,
        sig_data,
        signer: key.to_string(),
        ..Default::default()
    };

    let signed_response = SignedResponse {
        id: response.id.clone(),
        thread: response.thread.clone(),
        connection_sig,
        please_ack: response.please_ack.clone(),
        timing: response.timing.clone(),
    };

    Ok(signed_response)
}

pub async fn decode_signed_connection_response(response: SignedResponse, their_vk: &str) -> VcxResult<Response> {
    let signature =
        base64::decode_config(&response.connection_sig.signature.as_bytes(), base64::URL_SAFE).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot decode ConnectionResponse: {:?}", err),
            )
        })?;

    let sig_data =
        base64::decode_config(&response.connection_sig.sig_data.as_bytes(), base64::URL_SAFE).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot decode ConnectionResponse: {:?}", err),
            )
        })?;

    if !verify(their_vk, &sig_data, &signature).await? {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "ConnectionResponse signature is invalid for original Invite recipient key",
        ));
    }

    if response.connection_sig.signer != their_vk {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "Signer declared in ConnectionResponse signed response is not matching the actual signer. Connection ",
        ));
    }

    let sig_data = &sig_data[8..];

    let connection: ConnectionData = serde_json::from_slice(sig_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, err.to_string()))?;

    Ok(Response {
        id: response.id,
        thread: response.thread,
        connection,
        please_ack: response.please_ack,
        timing: response.timing,
    })
}

pub async fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() {
        return Ok(true);
    }

    crypto::verify(vk, msg, signature).await.map_err(VcxError::from)
}

pub async fn pack_message(
    wallet_handle: WalletHandle,
    sender_vk: Option<&str>,
    receiver_keys: &str,
    msg: &[u8],
) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(msg.to_vec());
    }

    crypto::pack_message(wallet_handle, msg, receiver_keys, sender_vk)
        .await
        .map_err(VcxError::from)
}

pub async fn unpack_message(wallet_handle: WalletHandle, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    crypto::unpack_message(wallet_handle, msg).await.map_err(VcxError::from)
}

pub async fn create_key(wallet_handle: WalletHandle, seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({ "seed": seed }).to_string();

    crypto::create_key(wallet_handle, Some(&key_json))
        .await
        .map_err(VcxError::from)
}
