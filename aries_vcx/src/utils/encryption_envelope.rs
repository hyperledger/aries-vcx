use vdrtools::future::TryFutureExt;
use vdrtools_sys::WalletHandle;

use agency_client::testing::mocking::AgencyMockDecrypted;

use messages::did_doc::DidDoc;
use crate::error::prelude::*;
use crate::global::settings;
use crate::indy::signing;
use crate::utils::constants;
use messages::a2a::A2AMessage;
use messages::forward::Forward;

#[derive(Debug)]
pub struct EncryptionEnvelope(pub Vec<u8>);

impl EncryptionEnvelope {
    pub async fn create(
        wallet_handle: WalletHandle,
        message: &A2AMessage,
        pw_verkey: Option<&str>,
        did_doc: &DidDoc,
    ) -> VcxResult<EncryptionEnvelope> {
        trace!(
            "EncryptionEnvelope::create >>> message: {:?}, pw_verkey: {:?}, did_doc: {:?}",
            message,
            pw_verkey,
            did_doc
        );

        if settings::indy_mocks_enabled() {
            return Ok(EncryptionEnvelope(vec![]));
        }

        EncryptionEnvelope::encrypt_for_pairwise(wallet_handle, message, pw_verkey, did_doc)
            .and_then(|message| async move {
                EncryptionEnvelope::wrap_into_forward_messages(wallet_handle, message, did_doc).await
            })
            .await
            .map(EncryptionEnvelope)
    }

    async fn encrypt_for_pairwise(
        wallet_handle: WalletHandle,
        message: &A2AMessage,
        pw_verkey: Option<&str>,
        did_doc: &DidDoc,
    ) -> VcxResult<Vec<u8>> {
        let message = match message {
            A2AMessage::Generic(message_) => message_.to_string(),
            message => json!(message).to_string(),
        };

        let receiver_keys = json!(did_doc.recipient_keys()).to_string();

        debug!(
            "Encrypting for pairwise; pw_verkey: {:?}, receiver_keys: {:?}",
            pw_verkey, receiver_keys
        );
        signing::pack_message(wallet_handle, pw_verkey, &receiver_keys, message.as_bytes()).await
    }

    async fn wrap_into_forward_messages(
        wallet_handle: WalletHandle,
        mut message: Vec<u8>,
        did_doc: &DidDoc,
    ) -> VcxResult<Vec<u8>> {
        let recipient_keys = did_doc.recipient_keys();
        let routing_keys = did_doc.routing_keys();

        let mut to = recipient_keys.get(0).map(String::from).ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidConnectionHandle,
            format!("Recipient Key not found in DIDDoc: {:?}", did_doc),
        ))?;

        for routing_key in routing_keys.iter() {
            message = EncryptionEnvelope::wrap_into_forward(wallet_handle, message, &to, routing_key).await?;
            to = routing_key.clone();
        }

        Ok(message)
    }

    async fn wrap_into_forward(
        wallet_handle: WalletHandle,
        message: Vec<u8>,
        to: &str,
        routing_key: &str,
    ) -> VcxResult<Vec<u8>> {
        let message = A2AMessage::Forward(Forward::new(to.to_string(), message)?);

        let message = json!(message).to_string();
        let receiver_keys = json!(vec![routing_key]).to_string();

        signing::pack_message(wallet_handle, None, &receiver_keys, message.as_bytes()).await
    }

    async fn _unpack_a2a_message(wallet_handle: WalletHandle, payload: Vec<u8>) -> VcxResult<(String, Option<String>)> {
        trace!(
            "EncryptionEnvelope::_unpack_a2a_message >>> processing payload of {} bytes",
            payload.len()
        );

        let unpacked_msg = signing::unpack_message(wallet_handle, &payload).await?;

        let msg_value: serde_json::Value = serde_json::from_slice(unpacked_msg.as_slice()).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize message: {}", err),
            )
        })?;

        let sender_vk = msg_value["sender_verkey"].as_str().map(String::from);

        let msg_string = msg_value["message"]
            .as_str()
            .ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                "Cannot find `message` field",
            ))?
            .to_string();

        Ok((msg_string, sender_vk))
    }

    // todo: we should use auth_unpack wherever possible
    pub async fn anon_unpack(wallet_handle: WalletHandle, payload: Vec<u8>) -> VcxResult<(A2AMessage, Option<String>)> {
        trace!(
            "EncryptionEnvelope::anon_unpack >>> processing payload of {} bytes",
            payload.len()
        );
        let (message, sender_vk) = if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::anon_unpack >>> returning decrypted mock message");
            (AgencyMockDecrypted::get_next_decrypted_message(), Some(constants::VERKEY.to_string()))
        } else {
            Self::_unpack_a2a_message(wallet_handle, payload).await?
        };
        let a2a_message = serde_json::from_str(&message).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?;
        Ok((a2a_message, sender_vk))
    }

    pub async fn auth_unpack(
        wallet_handle: WalletHandle,
        payload: Vec<u8>,
        expected_vk: &str,
    ) -> VcxResult<A2AMessage> {
        trace!(
            "EncryptionEnvelope::auth_unpack >>> processing payload of {} bytes, expected_vk: {}",
            payload.len(),
            expected_vk
        );

        let message = if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::auth_unpack >>> returning decrypted mock message");
            AgencyMockDecrypted::get_next_decrypted_message()
        } else {
            let (a2a_message, sender_vk) = Self::_unpack_a2a_message(wallet_handle, payload).await?;
            trace!("anon_unpack >> a2a_msg: {:?}, sender_vk: {:?}", a2a_message, sender_vk);

            match sender_vk {
                Some(sender_vk) => {
                    if sender_vk != expected_vk {
                        error!(
                            "auth_unpack  sender_vk != expected_vk.... sender_vk: {}, expected_vk: {}",
                            sender_vk, expected_vk
                        );
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Message did not pass authentication check. Expected sender verkey was {}, but actually was {}", expected_vk, sender_vk)));
                    }
                }
                None => {
                    error!("auth_unpack  message was authcrypted");
                    return Err(VcxError::from_msg(
                        VcxErrorKind::InvalidJson,
                        "Can't authenticate message because it was anoncrypted.",
                    ));
                }
            }
            a2a_message
        };
        let a2a_message = serde_json::from_str(&message).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?;
        Ok(a2a_message)
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::did_doc::test_utils::*;
    use crate::indy::signing::create_key;
    use crate::indy::utils::test_setup;
    use crate::indy::utils::test_setup::create_trustee_key;
    use messages::ack::test_utils::_ack;
    use crate::utils::devsetup::SetupEmpty;

    use super::*;

    #[tokio::test]
    async fn test_encryption_envelope_works_for_no_keys() {
        SetupEmpty::init();
        let setup = test_setup::setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wallet_handle).await;

        let message = A2AMessage::Ack(_ack());

        let res =
            EncryptionEnvelope::create(setup.wallet_handle, &message, Some(&trustee_key), &DidDoc::default()).await;
        assert_eq!(res.unwrap_err().kind(), VcxErrorKind::InvalidLibindyParam);
    }

    #[tokio::test]
    async fn test_encryption_envelope_works_for_recipient_only() {
        SetupEmpty::init();
        let setup = test_setup::setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wallet_handle).await;

        let message = A2AMessage::Ack(_ack());

        let envelope = EncryptionEnvelope::create(
            setup.wallet_handle,
            &message,
            Some(&trustee_key),
            &_did_doc_empty_routing(),
        )
        .await
        .unwrap();
        assert_eq!(
            message,
            EncryptionEnvelope::anon_unpack(setup.wallet_handle, envelope.0)
                .await
                .unwrap().0
        );
    }

    #[tokio::test]
    async fn test_encryption_envelope_works_for_routing_keys() {
        SetupEmpty::init();
        let setup = test_setup::setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wallet_handle).await;

        let key_1 = create_key(setup.wallet_handle, None).await.unwrap();
        let key_2 = create_key(setup.wallet_handle, None).await.unwrap();

        let mut did_doc = DidDoc::default();
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(vec![key_1.clone(), key_2.clone()]);

        let ack = A2AMessage::Ack(_ack());

        let envelope = EncryptionEnvelope::create(setup.wallet_handle, &ack, Some(&trustee_key), &did_doc)
            .await
            .unwrap();

        let message_1 = EncryptionEnvelope::anon_unpack(setup.wallet_handle, envelope.0)
            .await
            .unwrap().0;

        let message_1 = match message_1 {
            A2AMessage::Forward(forward) => {
                assert_eq!(key_1, forward.to);
                serde_json::to_vec(&forward.msg).unwrap()
            }
            _ => return assert!(false),
        };

        let message_2 = EncryptionEnvelope::anon_unpack(setup.wallet_handle, message_1)
            .await
            .unwrap().0;

        let message_2 = match message_2 {
            A2AMessage::Forward(forward) => {
                assert_eq!(_key_1(), forward.to);
                serde_json::to_vec(&forward.msg).unwrap()
            }
            _ => return assert!(false),
        };

        assert_eq!(
            ack,
            EncryptionEnvelope::anon_unpack(setup.wallet_handle, message_2)
                .await
                .unwrap().0
        );
    }

    #[tokio::test]
    async fn test_auth_unpack_message_should_succeed_if_sender_key_matches_expectation() {
        SetupEmpty::init();
        let recipient_wallet = test_setup::setup_wallet().await;
        let recipient_key = test_setup::create_key(recipient_wallet.wallet_handle).await;

        let sender_wallet = test_setup::setup_wallet().await;
        let sender_key = test_setup::create_key(sender_wallet.wallet_handle).await;

        let mut did_doc = DidDoc::default();
        did_doc.set_recipient_keys(vec![recipient_key]);

        let ack = A2AMessage::Ack(_ack());
        let envelope = EncryptionEnvelope::create(sender_wallet.wallet_handle, &ack, Some(&sender_key), &did_doc)
            .await
            .unwrap();
        let _message_1 = EncryptionEnvelope::auth_unpack(recipient_wallet.wallet_handle, envelope.0, &sender_key)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_auth_unpack_message_should_fail_if_sender_key_does_not_match_expectation() {
        SetupEmpty::init();
        let recipient_wallet = test_setup::setup_wallet().await;
        let recipient_key = test_setup::create_key(recipient_wallet.wallet_handle).await;

        let sender_wallet = test_setup::setup_wallet().await;
        let sender_key_1 = test_setup::create_key(sender_wallet.wallet_handle).await;
        let sender_key_2 = test_setup::create_key(sender_wallet.wallet_handle).await;

        let mut did_doc = DidDoc::default();
        did_doc.set_recipient_keys(vec![recipient_key]);

        let ack = A2AMessage::Ack(_ack());
        let envelope = EncryptionEnvelope::create(sender_wallet.wallet_handle, &ack, Some(&sender_key_2), &did_doc)
            .await
            .unwrap();
        let result = EncryptionEnvelope::auth_unpack(recipient_wallet.wallet_handle, envelope.0, &sender_key_1).await;
        assert!(result.is_err());
    }
}
