use std::sync::Arc;

use agency_client::testing::mocking::AgencyMockDecrypted;
use futures::TryFutureExt;
use messages::{a2a::A2AMessage, diddoc::aries::diddoc::AriesDidDoc, protocols::routing::forward::Forward};

use crate::{errors::error::prelude::*, global::settings, plugins::wallet::base_wallet::BaseWallet, utils::constants};

#[derive(Debug)]
pub struct EncryptionEnvelope(pub Vec<u8>);

impl EncryptionEnvelope {
    pub async fn create(
        wallet: &Arc<dyn BaseWallet>,
        message: &A2AMessage,
        pw_verkey: Option<&str>,
        did_doc: &AriesDidDoc,
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

        EncryptionEnvelope::encrypt_for_pairwise(wallet, message, pw_verkey, did_doc)
            .and_then(|message| async move {
                EncryptionEnvelope::wrap_into_forward_messages(wallet, message, did_doc).await
            })
            .await
            .map(EncryptionEnvelope)
    }

    async fn encrypt_for_pairwise(
        wallet: &Arc<dyn BaseWallet>,
        message: &A2AMessage,
        pw_verkey: Option<&str>,
        did_doc: &AriesDidDoc,
    ) -> VcxResult<Vec<u8>> {
        let message = match message {
            A2AMessage::Generic(message_) => message_.to_string(),
            message => json!(message).to_string(),
        };

        let receiver_keys = json!(did_doc.recipient_keys()?).to_string();

        debug!(
            "Encrypting for pairwise; pw_verkey: {:?}, receiver_keys: {:?}",
            pw_verkey, receiver_keys
        );

        wallet.pack_message(pw_verkey, &receiver_keys, message.as_bytes()).await
    }

    async fn wrap_into_forward_messages(
        wallet: &Arc<dyn BaseWallet>,
        mut message: Vec<u8>,
        did_doc: &AriesDidDoc,
    ) -> VcxResult<Vec<u8>> {
        let recipient_keys = did_doc.recipient_keys()?;
        let routing_keys = did_doc.routing_keys();

        let mut to = recipient_keys.get(0).map(String::from).ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            format!("Recipient Key not found in DIDDoc: {:?}", did_doc),
        ))?;

        for routing_key in routing_keys.iter() {
            message = EncryptionEnvelope::wrap_into_forward(wallet, message, &to, routing_key).await?;
            to = routing_key.clone();
        }

        Ok(message)
    }

    async fn wrap_into_forward(
        wallet: &Arc<dyn BaseWallet>,
        message: Vec<u8>,
        to: &str,
        routing_key: &str,
    ) -> VcxResult<Vec<u8>> {
        let message = A2AMessage::Forward(Forward::new(to.to_string(), message)?);

        let message = json!(message).to_string();
        let receiver_keys = json!(vec![routing_key]).to_string();

        wallet.pack_message(None, &receiver_keys, message.as_bytes()).await
    }

    async fn _unpack_a2a_message(
        wallet: &Arc<dyn BaseWallet>,
        payload: Vec<u8>,
    ) -> VcxResult<(String, Option<String>)> {
        trace!(
            "EncryptionEnvelope::_unpack_a2a_message >>> processing payload of {} bytes",
            payload.len()
        );

        let unpacked_msg = wallet.unpack_message(&payload).await?;

        let msg_value: serde_json::Value = serde_json::from_slice(unpacked_msg.as_slice()).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize message: {}", err),
            )
        })?;

        let sender_vk = msg_value["sender_verkey"].as_str().map(String::from);

        let msg_string = msg_value["message"]
            .as_str()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                "Cannot find `message` field",
            ))?
            .to_string();

        Ok((msg_string, sender_vk))
    }

    // todo: we should use auth_unpack wherever possible
    pub async fn anon_unpack(
        wallet: &Arc<dyn BaseWallet>,
        payload: Vec<u8>,
    ) -> VcxResult<(A2AMessage, Option<String>)> {
        trace!(
            "EncryptionEnvelope::anon_unpack >>> processing payload of {} bytes",
            payload.len()
        );
        let (message, sender_vk) = if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::anon_unpack >>> returning decrypted mock message");
            (
                AgencyMockDecrypted::get_next_decrypted_message(),
                Some(constants::VERKEY.to_string()),
            )
        } else {
            Self::_unpack_a2a_message(wallet, payload).await?
        };
        let a2a_message = serde_json::from_str(&message).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?;
        Ok((a2a_message, sender_vk))
    }

    pub async fn auth_unpack(
        wallet: &Arc<dyn BaseWallet>,
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
            let (a2a_message, sender_vk) = Self::_unpack_a2a_message(wallet, payload).await?;
            trace!("anon_unpack >> a2a_msg: {:?}, sender_vk: {:?}", a2a_message, sender_vk);

            match sender_vk {
                Some(sender_vk) => {
                    if sender_vk != expected_vk {
                        error!(
                            "auth_unpack  sender_vk != expected_vk.... sender_vk: {}, expected_vk: {}",
                            sender_vk, expected_vk
                        );
                        return Err(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidJson,
                            format!(
                                "Message did not pass authentication check. Expected sender verkey was {}, but \
                                 actually was {}",
                                expected_vk, sender_vk
                            ),
                        ));
                    }
                }
                None => {
                    error!("auth_unpack  message was authcrypted");
                    return Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidJson,
                        "Can't authenticate message because it was anoncrypted.",
                    ));
                }
            }
            a2a_message
        };
        let a2a_message = serde_json::from_str(&message).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?;
        Ok(a2a_message)
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use messages::{concepts::ack::test_utils::_ack, diddoc::aries::diddoc::test_utils::*};

    use super::*;
    use crate::{
        common::test_utils::{create_key, create_trustee_key, indy_handles_to_profile},
        indy::utils::test_setup,
        utils::devsetup::SetupEmpty,
    };

    #[tokio::test]
    async fn test_encryption_envelope_works_for_no_keys() {
        SetupEmpty::init();
        test_setup::with_wallet(|wallet_handle| async move {
            let profile = indy_handles_to_profile(wallet_handle, 0);
            let trustee_key = create_trustee_key(&profile).await;

            let message = A2AMessage::Ack(_ack());

            let res = EncryptionEnvelope::create(
                &profile.inject_wallet(),
                &message,
                Some(&trustee_key),
                &AriesDidDoc::default(),
            )
            .await;
            assert_eq!(res.unwrap_err().kind(), AriesVcxErrorKind::InvalidLibindyParam);
        })
        .await;
    }

    #[tokio::test]
    async fn test_encryption_envelope_works_for_recipient_only() {
        SetupEmpty::init();
        test_setup::with_wallet(|wallet_handle| async move {
            let profile = indy_handles_to_profile(wallet_handle, 0);
            let trustee_key = create_trustee_key(&profile).await;

            let message = A2AMessage::Ack(_ack());

            let envelope = EncryptionEnvelope::create(
                &profile.inject_wallet(),
                &message,
                Some(&trustee_key),
                &_did_doc_empty_routing(),
            )
            .await
            .unwrap();
            assert_eq!(
                message,
                EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), envelope.0)
                    .await
                    .unwrap()
                    .0
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_encryption_envelope_works_for_routing_keys() {
        SetupEmpty::init();
        test_setup::with_wallet(|wallet_handle| async move {
            let profile = indy_handles_to_profile(wallet_handle, 0);
            let trustee_key = create_trustee_key(&profile).await;

            let key_1 = create_key(&profile).await;
            let key_2 = create_key(&profile).await;

            let mut did_doc = AriesDidDoc::default();
            did_doc.set_service_endpoint(_service_endpoint());
            did_doc.set_recipient_keys(_recipient_keys());
            did_doc.set_routing_keys(vec![key_1.clone(), key_2.clone()]);

            let ack = A2AMessage::Ack(_ack());

            let envelope = EncryptionEnvelope::create(&profile.inject_wallet(), &ack, Some(&trustee_key), &did_doc)
                .await
                .unwrap();

            let message_1 = EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), envelope.0)
                .await
                .unwrap()
                .0;

            let message_1 = match message_1 {
                A2AMessage::Forward(forward) => {
                    assert_eq!(key_1, forward.to);
                    serde_json::to_vec(&forward.msg).unwrap()
                }
                _ => return assert!(false),
            };

            let message_2 = EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), message_1)
                .await
                .unwrap()
                .0;

            let message_2 = match message_2 {
                A2AMessage::Forward(forward) => {
                    assert_eq!(_key_1(), forward.to);
                    serde_json::to_vec(&forward.msg).unwrap()
                }
                _ => return assert!(false),
            };

            assert_eq!(
                ack,
                EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), message_2)
                    .await
                    .unwrap()
                    .0
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_auth_unpack_message_should_succeed_if_sender_key_matches_expectation() {
        SetupEmpty::init();

        test_setup::with_wallet(|recipient_wallet| async move {
            let recipient_profile = indy_handles_to_profile(recipient_wallet, 0);
            let recipient_key = test_setup::create_key(recipient_wallet).await;

            test_setup::with_wallet(|sender_wallet| async move {
                let sender_profile = indy_handles_to_profile(sender_wallet, 0);
                let sender_key = test_setup::create_key(sender_wallet).await;

                let mut did_doc = AriesDidDoc::default();
                did_doc.set_recipient_keys(vec![recipient_key]);

                let ack = A2AMessage::Ack(_ack());
                let envelope =
                    EncryptionEnvelope::create(&sender_profile.inject_wallet(), &ack, Some(&sender_key), &did_doc)
                        .await
                        .unwrap();
                let _message_1 =
                    EncryptionEnvelope::auth_unpack(&recipient_profile.inject_wallet(), envelope.0, &sender_key)
                        .await
                        .unwrap();
            })
            .await;
        })
        .await;
    }

    #[tokio::test]
    async fn test_auth_unpack_message_should_fail_if_sender_key_does_not_match_expectation() {
        let _setup = SetupEmpty::init();

        test_setup::with_wallet(|recipient_wallet| async move {
            let recipient_profile = indy_handles_to_profile(recipient_wallet, 0);
            let recipient_key = test_setup::create_key(recipient_wallet).await;

            test_setup::with_wallet(|sender_wallet| async move {
                let sender_profile = indy_handles_to_profile(sender_wallet, 0);
                let sender_key_1 = test_setup::create_key(sender_wallet).await;
                let sender_key_2 = test_setup::create_key(sender_wallet).await;

                let mut did_doc = AriesDidDoc::default();

                did_doc.set_recipient_keys(vec![recipient_key]);

                let ack = A2AMessage::Ack(_ack());
                let envelope =
                    EncryptionEnvelope::create(&sender_profile.inject_wallet(), &ack, Some(&sender_key_2), &did_doc)
                        .await
                        .unwrap();

                let result =
                    EncryptionEnvelope::auth_unpack(&recipient_profile.inject_wallet(), envelope.0, &sender_key_1)
                        .await;

                assert!(result.is_err());
            })
            .await;
        })
        .await;
    }
}
