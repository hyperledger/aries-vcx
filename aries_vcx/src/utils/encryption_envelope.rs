use agency_client::mocking::AgencyMockDecrypted;
use indy::future::TryFutureExt;

use crate::error::prelude::*;
use crate::libindy::utils::crypto;
use crate::messages::a2a::A2AMessage;
use crate::messages::connection::did_doc::DidDoc;
use crate::messages::forward::Forward;
use crate::settings;

#[derive(Debug)]
pub struct EncryptionEnvelope(pub Vec<u8>);

impl EncryptionEnvelope {
    pub async fn create(message: &A2AMessage,
                  pw_verkey: Option<&str>,
                  did_doc: &DidDoc) -> VcxResult<EncryptionEnvelope> {
        trace!("EncryptionEnvelope::create >>> message: {:?}, pw_verkey: {:?}, did_doc: {:?}", message, pw_verkey, did_doc);

        if settings::indy_mocks_enabled() { return Ok(EncryptionEnvelope(vec![])); }

        EncryptionEnvelope::encrypt_for_pairwise(message, pw_verkey, did_doc)
            .and_then(|message| async move { EncryptionEnvelope::wrap_into_forward_messages(message, did_doc).await })
            .await
            .map(|message| EncryptionEnvelope(message))
    }

    async fn encrypt_for_pairwise(message: &A2AMessage,
                            pw_verkey: Option<&str>,
                            did_doc: &DidDoc) -> VcxResult<Vec<u8>> {
        let message = match message {
            A2AMessage::Generic(message_) => message_.to_string(),
            message => json!(message).to_string()
        };

        let receiver_keys = json!(did_doc.recipient_keys()).to_string();

        debug!("Encrypting for pairwise; pw_verkey: {:?}, receiver_keys: {:?}", pw_verkey, receiver_keys);
        crypto::pack_message(pw_verkey, &receiver_keys, message.as_bytes()).await
    }

    async fn wrap_into_forward_messages(mut message: Vec<u8>,
                                  did_doc: &DidDoc) -> VcxResult<Vec<u8>> {
        let (recipient_keys, routing_keys) = did_doc.resolve_keys();

        let mut to = recipient_keys.get(0)
            .map(String::from)
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Recipient Key not found in DIDDoc: {:?}", did_doc)))?;

        for routing_key in routing_keys.iter() {
            message = EncryptionEnvelope::wrap_into_forward(message, &to, &routing_key).await?;
            to = routing_key.clone();
        }

        Ok(message)
    }

    async fn wrap_into_forward(message: Vec<u8>,
                         to: &str,
                         routing_key: &str) -> VcxResult<Vec<u8>> {
        let message = A2AMessage::Forward(Forward::new(to.to_string(), message)?);

        let message = json!(message).to_string();
        let receiver_keys = json!(vec![routing_key]).to_string();

        crypto::pack_message(None, &receiver_keys, message.as_bytes()).await
    }

    async fn _unpack_a2a_message(payload: Vec<u8>) -> VcxResult<(String, Option<String>)> {
        trace!("EncryptionEnvelope::_unpack_a2a_message >>> processing payload of {} bytes", payload.len());

        let unpacked_msg = crypto::unpack_message(&payload).await?;

        let msg_value: serde_json::Value = serde_json::from_slice(unpacked_msg.as_slice())
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize message: {}", err)))?;

        let sender_vk = msg_value["sender_verkey"].as_str().map(String::from);

        let msg_string = msg_value["message"].as_str()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, "Cannot find `message` field"))?.to_string();

        Ok((msg_string, sender_vk))
    }

    // todo: we should use auth_unpack wherever possible
    pub async fn anon_unpack(payload: Vec<u8>) -> VcxResult<A2AMessage> {
        trace!("EncryptionEnvelope::anon_unpack >>> processing payload of {} bytes", payload.len());
        let message = if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::anon_unpack >>> returning decrypted mock message");
            AgencyMockDecrypted::get_next_decrypted_message()
        } else {
            let (a2a_message, _sender_vk) = Self::_unpack_a2a_message(payload).await?;
            a2a_message
        };
        let a2a_message = serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))?;
        Ok(a2a_message)
    }

    pub async fn auth_unpack(payload: Vec<u8>, expected_vk: &str) -> VcxResult<A2AMessage> {
        trace!("EncryptionEnvelope::auth_unpack >>> processing payload of {} bytes, expected_vk: {}", payload.len(), expected_vk);

        let message = if AgencyMockDecrypted::has_decrypted_mock_messages() {
            warn!("EncryptionEnvelope::auth_unpack >>> returning decrypted mock message");
            AgencyMockDecrypted::get_next_decrypted_message()
        } else {
            let (a2a_message, sender_vk) = Self::_unpack_a2a_message(payload).await?;
            trace!("anon_unpack >> a2a_msg: {:?}, sender_vk: {:?}", a2a_message, sender_vk);

            match sender_vk {
                Some(sender_vk) => {
                    if sender_vk != expected_vk {
                        error!("auth_unpack  sender_vk != expected_vk.... sender_vk: {}, expected_vk: {}", sender_vk, expected_vk);
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                      format!("Message did not pass authentication check. Expected sender verkey was {}, but actually was {}", expected_vk, sender_vk))
                        );
                    }
                }
                None => {
                    error!("auth_unpack  message was authcrypted");
                    return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Can't authenticate message because it was anoncrypted."));
                }
            }
            a2a_message
        };
        let a2a_message = serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))?;
        Ok(a2a_message)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::libindy::utils::{test_setup, wallet};
    use crate::libindy::utils::crypto::create_key;
    use crate::libindy::utils::test_setup::create_trustee_key;
    use crate::messages::ack::test_utils::_ack;
    use crate::messages::connection::did_doc::test_utils::*;
    use crate::utils::devsetup::SetupEmpty;

    use super::*;

    fn _setup() {
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_encryption_envelope_works_for_no_keys() {
        _setup();
        let setup = test_setup::setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wh).await;

        let message = A2AMessage::Ack(_ack());

        let res = EncryptionEnvelope::create(&message, Some(&trustee_key), &DidDoc::default()).await;
        assert_eq!(res.unwrap_err().kind(), VcxErrorKind::InvalidLibindyParam);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_encryption_envelope_works_for_recipient_only() {
        _setup();
        let setup = test_setup::setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wh).await;

        let message = A2AMessage::Ack(_ack());

        let envelope = EncryptionEnvelope::create(&message, Some(&trustee_key), &_did_doc_4()).await.unwrap();
        assert_eq!(message, EncryptionEnvelope::anon_unpack(envelope.0).await.unwrap());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_encryption_envelope_works_for_routing_keys() {
        _setup();
        let setup = test_setup::setup_wallet().await;
        let trustee_key = create_trustee_key(setup.wh).await;

        let key_1 = create_key(None).await.unwrap();
        let key_2 = create_key(None).await.unwrap();

        let mut did_doc = DidDoc::default();
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_keys(_recipient_keys(), vec![key_1.clone(), key_2.clone()]);

        let ack = A2AMessage::Ack(_ack());

        let envelope = EncryptionEnvelope::create(&ack, Some(&trustee_key), &did_doc).await.unwrap();

        let message_1 = EncryptionEnvelope::anon_unpack(envelope.0).await.unwrap();

        let message_1 = match message_1 {
            A2AMessage::Forward(forward) => {
                assert_eq!(key_1, forward.to);
                serde_json::to_vec(&forward.msg).unwrap()
            }
            _ => return assert!(false)
        };

        let message_2 = EncryptionEnvelope::anon_unpack(message_1).await.unwrap();

        let message_2 = match message_2 {
            A2AMessage::Forward(forward) => {
                assert_eq!(_key_1(), forward.to);
                serde_json::to_vec(&forward.msg).unwrap()
            }
            _ => return assert!(false)
        };

        assert_eq!(ack, EncryptionEnvelope::anon_unpack(message_2).await.unwrap());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_auth_unpack_message_should_succeed_if_sender_key_matches_expectation() {
        SetupEmpty::init();
        _setup();
        let recipient_wallet = test_setup::setup_wallet().await;
        let recipient_key = test_setup::create_key(recipient_wallet.wh).await;

        let sender_wallet = test_setup::setup_wallet().await;
        let sender_key = test_setup::create_key(sender_wallet.wh).await;

        let mut did_doc = DidDoc::default();
        did_doc.set_keys(vec![recipient_key], vec![]);

        let ack = A2AMessage::Ack(_ack());

        wallet::set_wallet_handle(sender_wallet.wh);
        let envelope = EncryptionEnvelope::create(&ack, Some(&sender_key), &did_doc).await.unwrap();

        wallet::set_wallet_handle(recipient_wallet.wh);
        let _message_1 = EncryptionEnvelope::auth_unpack(envelope.0, &sender_key).await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_auth_unpack_message_should_fail_if_sender_key_does_not_match_expectation() {
        SetupEmpty::init();
        _setup();
        let recipient_wallet = test_setup::setup_wallet().await;
        let recipient_key = test_setup::create_key(recipient_wallet.wh).await;

        let sender_wallet = test_setup::setup_wallet().await;
        let sender_key_1 = test_setup::create_key(sender_wallet.wh).await;
        let sender_key_2 = test_setup::create_key(sender_wallet.wh).await;

        let mut did_doc = DidDoc::default();
        did_doc.set_keys(vec![recipient_key], vec![]);

        let ack = A2AMessage::Ack(_ack());

        wallet::set_wallet_handle(sender_wallet.wh);
        let envelope = EncryptionEnvelope::create(&ack, Some(&sender_key_2), &did_doc).await.unwrap();

        wallet::set_wallet_handle(recipient_wallet.wh);
        let result = EncryptionEnvelope::auth_unpack(envelope.0, &sender_key_1).await;
        assert!(result.is_err());
    }
}
