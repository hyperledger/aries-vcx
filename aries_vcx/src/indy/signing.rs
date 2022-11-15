use vdrtools::crypto;
use vdrtools_sys::WalletHandle;

use crate::error::prelude::*;
use crate::global::settings;

pub async fn sign(wallet_handle: WalletHandle, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    crypto::sign(wallet_handle, my_vk, msg).await.map_err(VcxError::from)
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

pub async fn unpack_message_to_string(wallet_handle: WalletHandle, msg: &[u8]) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(String::new());
    }

    String::from_utf8(
        crypto::unpack_message(wallet_handle, &msg)
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

pub async fn create_key(wallet_handle: WalletHandle, seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({ "seed": seed }).to_string();

    crypto::create_key(wallet_handle, Some(&key_json))
        .await
        .map_err(VcxError::from)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::did_doc::test_utils::*;
    use crate::indy::utils::test_setup::{create_trustee_key, setup_wallet};
    use messages::connection::response::test_utils::{_did, _response, _thread_id};
    use crate::utils::devsetup::SetupEmpty;

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
        let setup = setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wallet_handle).await;
        let signed_response: SignedResponse = sign_connection_response(setup.wallet_handle, &trustee_key, _response()).await.unwrap();
        assert_eq!(_response(), decode_signed_connection_response(signed_response, &trustee_key).await.unwrap());
    }

    #[tokio::test]
    async fn test_decode_returns_error_if_signer_differs() {
        SetupEmpty::init();
        let setup = setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wallet_handle).await;
        let mut signed_response: SignedResponse = sign_connection_response(setup.wallet_handle, &trustee_key, _response()).await.unwrap();
        signed_response.connection_sig.signer = String::from("AAAAAAAAAAAAAAAAXkaJdrQejfztN4XqdsiV4ct3LXKL");
        decode_signed_connection_response(signed_response, &trustee_key).await.unwrap_err();
    }
}
