use agency_client::testing::mocking::AgencyMockDecrypted;
use aries_vcx_core::{global::settings::VERKEY, wallet::base_wallet::BaseWallet};
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    msg_fields::protocols::routing::{Forward, ForwardContent},
    AriesMessage,
};
use uuid::Uuid;

use crate::errors::error::prelude::*;

#[derive(Debug)]
pub struct EncryptionEnvelope(pub Vec<u8>);

impl EncryptionEnvelope {
    /// Create an Encryption Envelope from a plaintext AriesMessage encoded as sequence of bytes.
    /// If did_doc includes routing_keys, then also wrap in appropriate layers of forward message.
    pub async fn create(
        wallet: &impl BaseWallet,
        message: &[u8],
        sender_vk: Option<&str>,
        did_doc: &AriesDidDoc,
    ) -> VcxResult<EncryptionEnvelope> {
        trace!(
            "EncryptionEnvelope::create >>> message: {:?}, pw_verkey: {:?}, did_doc: {:?}",
            message,
            sender_vk,
            did_doc
        );

        let recipient_key = did_doc.recipient_keys()?
            .first()
            .cloned()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!("No recipient key found in DIDDoc: {:?}", did_doc),
            ))?;
        let routing_keys = did_doc.routing_keys();
        let message = EncryptionEnvelope::encrypt_for_pairwise(
            wallet,
            message,
            sender_vk,
            recipient_key.clone(),
        )
        .await?;
        EncryptionEnvelope::wrap_into_forward_messages(
            wallet,
            message,
            recipient_key,
            routing_keys,
        )
        .await
        .map(EncryptionEnvelope)
    }

    async fn encrypt_for_pairwise(
        wallet: &impl BaseWallet,
        message: &[u8],
        sender_vk: Option<&str>,
        recipient_key: String,
    ) -> VcxResult<Vec<u8>> {
        debug!(
            "Encrypting for pairwise; sender_vk: {:?}, recipient_key: {}",
            sender_vk, recipient_key
        );
        let recipient_keys = json!([recipient_key.clone()]).to_string();
        wallet
            .pack_message(sender_vk, &recipient_keys, message)
            .await
            .map_err(|err| err.into())
    }

    async fn wrap_into_forward_messages(
        wallet: &impl BaseWallet,
        mut message: Vec<u8>,
        recipient_key: String,
        routing_keys: Vec<String>,
    ) -> VcxResult<Vec<u8>> {
        let mut forward_to_key = recipient_key;

        for routing_key in routing_keys.iter() {
            debug!(
                "Wrapping message in forward message; forward_to_key: {}, routing_key: {}",
                forward_to_key, routing_key
            );
            message = EncryptionEnvelope::wrap_into_forward(
                wallet,
                message,
                &forward_to_key,
                routing_key,
            )
                .await?;
            forward_to_key = routing_key.clone();
        }
        Ok(message)
    }

    async fn wrap_into_forward(
        wallet: &impl BaseWallet,
        message: Vec<u8>,
        forward_to_key: &str,
        routing_key: &str,
    ) -> VcxResult<Vec<u8>> {
        let content = ForwardContent::builder()
            .to(forward_to_key.to_string())
            .msg(serde_json::from_slice(&message)?)
            .build();

        let message: Forward = Forward::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .build();

        let message = json!(AriesMessage::from(message)).to_string();
        let receiver_keys = json!(vec![routing_key]).to_string();

        wallet
            .pack_message(None, &receiver_keys, message.as_bytes())
            .await
            .map_err(|err| err.into())
    }

    async fn _unpack_a2a_message(
        wallet: &impl BaseWallet,
        payload: Vec<u8>,
    ) -> VcxResult<(String, Option<String>)> {
        trace!(
            "EncryptionEnvelope::_unpack_a2a_message >>> processing payload of {} bytes",
            payload.len()
        );

        let unpacked_msg = wallet.unpack_message(&payload).await?;

        let sender_vk = unpacked_msg.sender_verkey;

        let msg_string = unpacked_msg.message;

        Ok((msg_string, sender_vk))
    }

    // todo: we should use auth_unpack wherever possible
    pub async fn anon_unpack(
        wallet: &impl BaseWallet,
        payload: Vec<u8>,
    ) -> VcxResult<(AriesMessage, Option<String>)> {
        trace!(
            "EncryptionEnvelope::anon_unpack >>> processing payload of {} bytes",
            payload.len()
        );
        let (message, sender_vk) = if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::anon_unpack >>> returning decrypted mock message");
            (
                AgencyMockDecrypted::get_next_decrypted_message(),
                Some(VERKEY.to_string()),
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
        wallet: &impl BaseWallet,
        payload: Vec<u8>,
        expected_vk: &str,
    ) -> VcxResult<AriesMessage> {
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
            trace!(
                "anon_unpack >> a2a_msg: {:?}, sender_vk: {:?}",
                a2a_message,
                sender_vk
            );

            match sender_vk {
                Some(sender_vk) => {
                    if sender_vk != expected_vk {
                        error!(
                            "auth_unpack  sender_vk != expected_vk.... sender_vk: {}, \
                             expected_vk: {}",
                            sender_vk, expected_vk
                        );
                        return Err(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidJson,
                            format!(
                                "Message did not pass authentication check. Expected sender \
                                 verkey was {}, but actually was {}",
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

// #[cfg(test)]
// pub mod unit_tests {
//     use crate::common::test_utils::{create_key, create_trustee_key, indy_handles_to_profile};
//     use crate::utils::devsetup::SetupEmpty;
//     use aries_vcx_core::indy::utils::test_setup;
//     use aries_vcx_core::INVALID_POOL_HANDLE;
//     use messages::concepts::ack::test_utils::_ack;
//     use messages::diddoc::aries::diddoc::test_utils::*;

//     use super::*;

//     #[tokio::test]
//     async fn test_encryption_envelope_works_for_no_keys() {
//         SetupEmpty::init();
//         test_setup::with_wallet(|wallet_handle| async move {
//             let profile = indy_handles_to_profile(wallet_handle, INVALID_POOL_HANDLE);
//             let trustee_key = create_trustee_key(&profile).await;

//             let message = A2AMessage::Ack(_ack());

//             let res = EncryptionEnvelope::create(
//                 &profile.inject_wallet(),
//                 &message,
//                 Some(&trustee_key),
//                 &AriesDidDoc::default(),
//             )
//             .await;
//             assert_eq!(res.unwrap_err().kind(), AriesVcxErrorKind::InvalidLibindyParam);
//         })
//         .await;
//     }

//     #[tokio::test]
//     async fn test_encryption_envelope_works_for_recipient_only() {
//         SetupEmpty::init();
//         test_setup::with_wallet(|wallet_handle| async move {
//             let profile = indy_handles_to_profile(wallet_handle, INVALID_POOL_HANDLE);
//             let trustee_key = create_trustee_key(&profile).await;

//             let message = A2AMessage::Ack(_ack());

//             let envelope = EncryptionEnvelope::create(
//                 &profile.inject_wallet(),
//                 &message,
//                 Some(&trustee_key),
//                 &_did_doc_empty_routing(),
//             )
//             .await
//             .unwrap();
//             assert_eq!(
//                 message,
//                 EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), envelope.0)
//                     .await
//                     .unwrap()
//                     .0
//             );
//         })
//         .await;
//     }

//     #[tokio::test]
//     async fn test_encryption_envelope_works_for_routing_keys() {
//         SetupEmpty::init();
//         test_setup::with_wallet(|wallet_handle| async move {
//             let profile = indy_handles_to_profile(wallet_handle, INVALID_POOL_HANDLE);
//             let trustee_key = create_trustee_key(&profile).await;

//             let key_1 = create_key(&profile).await;
//             let key_2 = create_key(&profile).await;

//             let mut did_doc = AriesDidDoc::default();
//             did_doc.set_service_endpoint(_service_endpoint());
//             did_doc.set_recipient_keys(_recipient_keys());
//             did_doc.set_routing_keys(vec![key_1.clone(), key_2.clone()]);

//             let ack = A2AMessage::Ack(_ack());

//             let envelope = EncryptionEnvelope::create(&profile.inject_wallet(), &ack,
// Some(&trustee_key), &did_doc)                 .await
//                 .unwrap();

//             let message_1 = EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), envelope.0)
//                 .await
//                 .unwrap()
//                 .0;

//             let message_1 = match message_1 {
//                 A2AMessage::Forward(forward) => {
//                     assert_eq!(key_1, forward.to);
//                     serde_json::to_vec(&forward.msg).unwrap()
//                 }
//                 _ => return assert!(false),
//             };

//             let message_2 = EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), message_1)
//                 .await
//                 .unwrap()
//                 .0;

//             let message_2 = match message_2 {
//                 A2AMessage::Forward(forward) => {
//                     assert_eq!(_key_1(), forward.to);
//                     serde_json::to_vec(&forward.msg).unwrap()
//                 }
//                 _ => return assert!(false),
//             };

//             assert_eq!(
//                 ack,
//                 EncryptionEnvelope::anon_unpack(&profile.inject_wallet(), message_2)
//                     .await
//                     .unwrap()
//                     .0
//             );
//         })
//         .await;
//     }

//     #[tokio::test]
//     async fn test_auth_unpack_message_should_succeed_if_sender_key_matches_expectation() {
//         SetupEmpty::init();

//         test_setup::with_wallet(|recipient_wallet| async move {
//             let recipient_profile = indy_handles_to_profile(recipient_wallet,
// INVALID_POOL_HANDLE);             let recipient_key =
// test_setup::create_key(recipient_wallet).await;

//             test_setup::with_wallet(|sender_wallet| async move {
//                 let sender_profile = indy_handles_to_profile(sender_wallet, INVALID_POOL_HANDLE);
//                 let sender_key = test_setup::create_key(sender_wallet).await;

//                 let mut did_doc = AriesDidDoc::default();
//                 did_doc.set_recipient_keys(vec![recipient_key]);

//                 let ack = A2AMessage::Ack(_ack());
//                 let envelope =
//                     EncryptionEnvelope::create(&sender_profile.inject_wallet(), &ack,
// Some(&sender_key), &did_doc)                         .await
//                         .unwrap();
//                 let _message_1 =
//                     EncryptionEnvelope::auth_unpack(&recipient_profile.inject_wallet(),
// envelope.0, &sender_key)                         .await
//                         .unwrap();
//             })
//             .await;
//         })
//         .await;
//     }

//     #[tokio::test]
//     async fn test_auth_unpack_message_should_fail_if_sender_key_does_not_match_expectation() {
//         let _setup = SetupEmpty::init();

//         test_setup::with_wallet(|recipient_wallet| async move {
//             let recipient_profile = indy_handles_to_profile(recipient_wallet,
// INVALID_POOL_HANDLE);             let recipient_key =
// test_setup::create_key(recipient_wallet).await;

//             test_setup::with_wallet(|sender_wallet| async move {
//                 let sender_profile = indy_handles_to_profile(sender_wallet, INVALID_POOL_HANDLE);
//                 let sender_key_1 = test_setup::create_key(sender_wallet).await;
//                 let sender_key_2 = test_setup::create_key(sender_wallet).await;

//                 let mut did_doc = AriesDidDoc::default();

//                 did_doc.set_recipient_keys(vec![recipient_key]);

//                 let ack = A2AMessage::Ack(_ack());
//                 let envelope =
//                     EncryptionEnvelope::create(&sender_profile.inject_wallet(), &ack,
// Some(&sender_key_2), &did_doc)                         .await
//                         .unwrap();

//                 let result =
//                     EncryptionEnvelope::auth_unpack(&recipient_profile.inject_wallet(),
// envelope.0, &sender_key_1)                         .await;

//                 assert!(result.is_err());
//             })
//             .await;
//         })
//         .await;
//     }
// }
