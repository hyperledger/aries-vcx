use std::sync::Arc;

use time;
use base64;

use crate::{error::prelude::*, plugins::wallet::base_wallet::BaseWallet, global::settings};
use messages::connection::response::{Response, SignedResponse, ConnectionSignature, ConnectionData};

async fn get_signature_data(wallet: &Arc<dyn BaseWallet>, data: String, key: &str) -> VcxResult<(Vec<u8>, Vec<u8>)> {
    let now: u64 = time::get_time().sec as u64;
    let mut sig_data = now.to_be_bytes().to_vec();
    sig_data.extend(data.as_bytes());
    
    let signature = wallet.sign(key, &sig_data).await?;
    
    Ok((signature, sig_data))
}

pub async fn sign_connection_response(wallet: &Arc<dyn BaseWallet>, key: &str, response: Response) -> VcxResult<SignedResponse> {
    let connection_data = response.get_connection_data();
    let (signature, sig_data) = get_signature_data(wallet, connection_data, key).await?;
    
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

pub async fn decode_signed_connection_response(wallet: &Arc<dyn BaseWallet>, response: SignedResponse, their_vk: &str) -> VcxResult<Response> {
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

    if !wallet.verify(their_vk, &sig_data, &signature).await? {
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

pub async fn unpack_message_to_string(wallet: &Arc<dyn BaseWallet>, msg: &[u8]) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(String::new());
    }

    String::from_utf8(
        wallet.unpack_message(&msg)
            .await
            .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Failed to unpack message"))?,
    )
    .map_err(|_| {
        VcxError::from_msg(
            VcxErrorKind::InvalidMessageFormat,
            "Failed to convert message to utf8 string",
        )
    })
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::did_doc::test_utils::*;
    use messages::connection::response::test_utils::{_did, _response, _thread_id};
    use crate::indy::utils::test_setup::with_wallet;
    use crate::utils::devsetup::SetupEmpty;
    use crate::common::test_utils::{create_trustee_key, indy_handles_to_profile};

    use super::*;

    #[test]
    fn test_response_build_works() {
        SetupEmpty::init();
        let response: Response = Response::default()
            .set_did(_did())
            .set_thread_id(&_thread_id())
            .set_service_endpoint(_service_endpoint())
            .set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_response(), response);
    }

    #[tokio::test]
    async fn test_response_encode_works() {
        SetupEmpty::init();
        with_wallet(|wallet_handle| async move {
        let profile = indy_handles_to_profile(wallet_handle, 0);
        let trustee_key = create_trustee_key(&profile).await;
        let signed_response: SignedResponse = sign_connection_response(&profile.inject_wallet(), &trustee_key, _response()).await.unwrap();
        assert_eq!(_response(), decode_signed_connection_response(&profile.inject_wallet(), signed_response, &trustee_key).await.unwrap());
        }).await;
    }

    #[tokio::test]
    async fn test_decode_returns_error_if_signer_differs() {
        SetupEmpty::init();
        with_wallet(|wallet_handle| async move {
        let profile = indy_handles_to_profile(wallet_handle, 0);
        let trustee_key = create_trustee_key(&profile).await;
        let mut signed_response: SignedResponse = sign_connection_response(&profile.inject_wallet(), &trustee_key, _response()).await.unwrap();
        signed_response.connection_sig.signer = String::from("AAAAAAAAAAAAAAAAXkaJdrQejfztN4XqdsiV4ct3LXKL");
        decode_signed_connection_response(&profile.inject_wallet(), signed_response, &trustee_key).await.unwrap_err();
        }).await;
    }
}
