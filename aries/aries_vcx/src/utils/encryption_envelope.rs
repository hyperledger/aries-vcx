use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_doc::schema::{did_doc::DidDocument, types::uri::Uri};
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    msg_fields::protocols::routing::{Forward, ForwardContent},
    AriesMessage,
};
use public_key::{Key, KeyType};
use uuid::Uuid;

use crate::{
    errors::error::prelude::*,
    utils::didcomm_utils::{
        get_ed25519_recipient_keys, get_ed25519_routing_keys, resolve_ed25519_key_agreement,
    },
};

#[derive(Debug)]
pub struct EncryptionEnvelope(pub Vec<u8>);

impl EncryptionEnvelope {
    pub async fn create_from_legacy(
        wallet: &impl BaseWallet,
        data: &[u8],
        sender_vk: Option<&str>,
        did_doc: &AriesDidDoc,
    ) -> VcxResult<EncryptionEnvelope> {
        trace!(
            "EncryptionEnvelope::create >>> data: {:?}, sender_vk: {:?}, did_doc: {:?}",
            data,
            sender_vk,
            did_doc
        );

        let recipient_key_base58 =
            did_doc
                .recipient_keys()?
                .first()
                .cloned()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    format!("No recipient key found in DIDDoc: {:?}", did_doc),
                ))?;

        let recipient_key = Key::from_base58(&recipient_key_base58, KeyType::Ed25519)?;
        let routing_keys = did_doc
            .routing_keys()
            .iter()
            .map(|routing_key| Key::from_base58(routing_key, KeyType::Ed25519))
            .collect::<Result<Vec<_>, _>>()?;
        let sender_key = sender_vk
            .map(|key| Key::from_base58(key, KeyType::Ed25519))
            .transpose()?;
        Self::create_from_keys(wallet, data, sender_key, recipient_key, routing_keys).await
    }

    /// Create encrypted message based on key agreement keys of our did document, counterparties
    /// did document and their specific service, identified by id, which must be part of their
    /// did document
    ///
    /// # Arguments
    ///
    /// * `our_did_doc` - Our did_document, which the counterparty should already be in possession
    ///   of
    /// * `their_did_doc` - The did document of the counterparty, the recipient of the encrypted
    ///   message
    /// * `their_service_id` - Id of service where message will be sent. The counterparty did
    ///   document must contain Service object identified with such value.
    pub async fn create(
        wallet: &impl BaseWallet,
        data: &[u8],
        our_did_doc: &DidDocument,
        their_did_doc: &DidDocument,
        their_service_id: &Uri,
    ) -> VcxResult<EncryptionEnvelope> {
        let sender_vk = resolve_ed25519_key_agreement(our_did_doc)?;

        let recipient_key = {
            let service_keys = get_ed25519_recipient_keys(their_did_doc, their_service_id)?;
            match service_keys.into_iter().next() {
                Some(key) => key,
                // as a backup, use the first key agreement key, or none
                None => resolve_ed25519_key_agreement(their_did_doc)?,
            }
        };
        let routing_keys = get_ed25519_routing_keys(their_did_doc, their_service_id)?;

        EncryptionEnvelope::create_from_keys(
            wallet,
            data,
            Some(sender_vk),
            recipient_key,
            routing_keys,
        )
        .await
    }

    pub async fn create_from_keys(
        wallet: &impl BaseWallet,
        data: &[u8],
        sender_vk: Option<Key>,
        recipient_key: Key,
        routing_keys: Vec<Key>,
    ) -> VcxResult<EncryptionEnvelope> {
        // Validate keys are Ed25519
        sender_vk
            .as_ref()
            .map(|key| key.validate_key_type(KeyType::Ed25519))
            .transpose()?;
        for key in routing_keys.iter().as_ref() {
            key.validate_key_type(KeyType::Ed25519)?;
        }

        let message = EncryptionEnvelope::encrypt_for_pairwise(
            wallet,
            data,
            sender_vk,
            recipient_key.validate_key_type(KeyType::Ed25519)?.clone(),
        )
        .await?;
        EncryptionEnvelope::wrap_into_forward_messages(wallet, message, recipient_key, routing_keys)
            .await
            .map(EncryptionEnvelope)
    }

    async fn encrypt_for_pairwise(
        wallet: &impl BaseWallet,
        data: &[u8],
        sender_vk: Option<Key>,
        recipient_key: Key,
    ) -> VcxResult<Vec<u8>> {
        debug!(
            "Encrypting for pairwise; sender_vk: {:?}, recipient_key: {}",
            sender_vk, recipient_key
        );

        let recipient_keys = vec![recipient_key];

        wallet
            .pack_message(sender_vk, recipient_keys, data)
            .await
            .map_err(|err| err.into())
    }

    async fn wrap_into_forward_messages(
        wallet: &impl BaseWallet,
        mut data: Vec<u8>,
        recipient_key: Key,
        routing_keys: Vec<Key>,
    ) -> VcxResult<Vec<u8>> {
        let mut forward_to_key = recipient_key;

        for routing_key in routing_keys {
            debug!(
                "Wrapping message in forward message; forward_to_key: {}, routing_key: {}",
                forward_to_key, routing_key
            );
            data = EncryptionEnvelope::wrap_into_forward(
                wallet,
                data,
                &forward_to_key,
                routing_key.clone(),
            )
            .await?;
            forward_to_key.clone_from(&routing_key);
        }
        Ok(data)
    }

    async fn wrap_into_forward(
        wallet: &impl BaseWallet,
        data: Vec<u8>,
        forward_to_key: &Key,
        routing_key: Key,
    ) -> VcxResult<Vec<u8>> {
        let content = ForwardContent::builder()
            .to(forward_to_key.base58())
            .msg(serde_json::from_slice(&data)?)
            .build();

        let message: Forward = Forward::builder()
            .id(Uuid::new_v4().to_string())
            .content(content)
            .build();

        let message = json!(AriesMessage::from(message)).to_string();

        let receiver_keys = vec![routing_key];

        wallet
            .pack_message(None, receiver_keys, message.as_bytes())
            .await
            .map_err(|err| err.into())
    }

    // Will unpack a message as either anoncrypt or authcrypt.
    async fn unpack_a2a_message(
        wallet: &impl BaseWallet,
        encrypted_data: &[u8],
    ) -> VcxResult<(String, Option<Key>, Key)> {
        trace!(
            "EncryptionEnvelope::unpack_a2a_message >>> processing payload of {} bytes",
            encrypted_data.len()
        );
        let unpacked_msg = wallet.unpack_message(encrypted_data).await?;
        let sender_key = unpacked_msg
            .sender_verkey
            .map(|key| Key::from_base58(&key, KeyType::Ed25519))
            .transpose()?;
        Ok((
            unpacked_msg.message,
            sender_key,
            Key::from_base58(&unpacked_msg.recipient_verkey, KeyType::Ed25519)?,
        ))
    }

    /// Unpacks an authcrypt or anoncrypt message returning the message, which is deserialized into an Aries message, as well as the sender key (if any -- anoncrypt does not return this) and the recipient key. Optionally takes expected_sender_vk, which does a comparison to ensure the sender key is the expected key.
    pub async fn unpack_aries_msg(
        wallet: &impl BaseWallet,
        encrypted_data: &[u8],
        expected_sender_vk: &Option<Key>,
    ) -> VcxResult<(AriesMessage, Option<Key>, Key)> {
        let (message, sender_vk, recipient_vk) =
            Self::unpack(wallet, encrypted_data, expected_sender_vk).await?;
        let a2a_message = serde_json::from_str(&message).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?;
        Ok((a2a_message, sender_vk, recipient_vk))
    }

    /// Unpacks an authcrypt or anoncrypt message returning the message, the sender key (if any -- anoncrypt does not return this), and the recipient key. Optionally takes expected_sender_vk, which does a comparison to ensure the sender key is the expected key.
    pub async fn unpack(
        wallet: &impl BaseWallet,
        encrypted_data: &[u8],
        expected_sender_vk: &Option<Key>,
    ) -> VcxResult<(String, Option<Key>, Key)> {
        trace!(
            "EncryptionEnvelope::anon_unpack >>> processing payload of {} bytes",
            encrypted_data.len()
        );
        let (a2a_message, sender_vk, recipient_vk) =
            Self::unpack_a2a_message(wallet, encrypted_data).await?;

        // If expected_sender_vk was provided and a sender_verkey exists, verify that they match
        if let Some(expected_key) = expected_sender_vk {
            match &sender_vk {
                Some(sender_vk) => {
                    if sender_vk != expected_key {
                        error!(
                        "auth_unpack  sender_vk != expected_sender_vk.... sender_vk: {}, expected_sender_vk: {}",
                        sender_vk, expected_key
                    );
                        return Err(AriesVcxError::from_msg(
                            AriesVcxErrorKind::AuthenticationError,
                            format!(
                            "Message did not pass authentication check. Expected sender verkey \
                             was {}, but actually was {}",
                             expected_key, sender_vk
                        ),
                        ));
                    }
                }
                None => {
                    error!("auth_unpack  message was authcrypted");
                    return Err(AriesVcxError::from_msg(
                        AriesVcxErrorKind::AuthenticationError,
                        "Can't authenticate message because it was anoncrypted.",
                    ));
                }
            }
        }
        Ok((a2a_message, sender_vk, recipient_vk))
    }
}

#[cfg(test)]
pub mod unit_tests {
    use aries_vcx_wallet::wallet::base_wallet::did_wallet::DidWallet;
    use serde_json::Value;
    use test_utils::devsetup::build_setup_profile;

    use super::*;

    #[tokio::test]
    async fn test_pack_unpack_anon() {
        let setup = build_setup_profile().await;
        let did_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();

        let data_original = "foobar";

        let envelope = EncryptionEnvelope::create_from_keys(
            &setup.wallet,
            data_original.as_bytes(),
            None,
            did_data.verkey().clone(),
            [].to_vec(),
        )
        .await
        .unwrap();

        let (data_unpacked, sender_verkey, _) =
            EncryptionEnvelope::unpack(&setup.wallet, &envelope.0, &None)
                .await
                .unwrap();

        assert_eq!(data_original, data_unpacked);
        assert!(sender_verkey.is_none());
    }

    #[tokio::test]
    async fn test_pack_unpack_auth() {
        let setup = build_setup_profile().await;
        let sender_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        let recipient_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();

        let sender_vk = sender_data.verkey().clone();
        let recipient_vk = recipient_data.verkey();

        let data_original = "foobar";

        let envelope = EncryptionEnvelope::create_from_keys(
            &setup.wallet,
            data_original.as_bytes(),
            Some(sender_vk.clone()),
            recipient_vk.clone(),
            [].to_vec(),
        )
        .await
        .unwrap();

        let (data_unpacked, _sender_vk_unpacked, _recipient_vk_unpacked) =
            EncryptionEnvelope::unpack(&setup.wallet, &envelope.0, &Some(sender_vk))
                .await
                .unwrap();

        assert_eq!(data_original, data_unpacked);
    }

    #[tokio::test]
    async fn test_pack_unpack_with_routing() {
        let setup = build_setup_profile().await;
        let sender_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        let recipient_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        let routing_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();

        let data_original = "foobar";

        let envelope = EncryptionEnvelope::create_from_keys(
            &setup.wallet,
            data_original.as_bytes(),
            Some(sender_data.verkey().clone()),
            recipient_data.verkey().clone(),
            [routing_data.verkey().clone()].to_vec(),
        )
        .await
        .unwrap();

        let (fwd_msg, _, _) = EncryptionEnvelope::unpack(&setup.wallet, &envelope.0, &None)
            .await
            .unwrap();
        let fwd_payload = serde_json::from_str::<Value>(&fwd_msg)
            .unwrap()
            .get("msg")
            .unwrap()
            .to_string();
        let (core_payload, _, _) =
            EncryptionEnvelope::unpack(&setup.wallet, fwd_payload.as_bytes(), &None)
                .await
                .unwrap();

        assert_eq!(data_original, core_payload);
    }

    #[tokio::test]
    async fn test_pack_unpack_unexpected_key_detection() {
        let setup = build_setup_profile().await;
        let alice_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        let bob_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        let recipient_data = setup
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();

        let data_original = "foobar";

        let envelope = EncryptionEnvelope::create_from_keys(
            &setup.wallet,
            data_original.as_bytes(),
            Some(bob_data.verkey().clone()), // bob trying to impersonate alice
            recipient_data.verkey().clone(),
            [].to_vec(),
        )
        .await
        .unwrap();

        let err = EncryptionEnvelope::unpack(
            &setup.wallet,
            &envelope.0,
            &Some(alice_data.verkey().clone()),
        )
        .await;
        assert!(err.is_err());
        assert_eq!(
            err.unwrap_err().kind(),
            AriesVcxErrorKind::AuthenticationError
        );
    }
}
