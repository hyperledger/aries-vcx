use std::sync::Arc;

use crate::{
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    testing::mocking::AgencyMockDecrypted,
    wallet::base_agency_client_wallet::BaseAgencyClientWallet,
};

#[derive(Debug)]
pub struct EncryptionEnvelope(pub Vec<u8>);

impl EncryptionEnvelope {
    async fn _unpack_a2a_message(
        wallet: Arc<dyn BaseAgencyClientWallet>,
        payload: Vec<u8>,
    ) -> AgencyClientResult<(String, Option<String>)> {
        trace!(
            "EncryptionEnvelope::_unpack_a2a_message >>> processing payload of {} bytes",
            payload.len()
        );

        let unpacked_msg = wallet.unpack_message(&payload).await?;

        let msg_value: ::serde_json::Value = ::serde_json::from_slice(unpacked_msg.as_slice()).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidJson,
                format!("Cannot deserialize message: {}", err),
            )
        })?;
        trace!("EncryptionEnvelope::_unpack_a2a_message >>> msg_value: {:?}", msg_value);

        let sender_vk = msg_value["sender_verkey"].as_str().map(String::from);
        trace!("EncryptionEnvelope::_unpack_a2a_message >>> sender_vk: {:?}", sender_vk);

        let msg_string = msg_value["message"]
            .as_str()
            .ok_or(AgencyClientError::from_msg(
                AgencyClientErrorKind::InvalidJson,
                "Cannot find `message` field",
            ))?
            .to_string();

        trace!(
            "EncryptionEnvelope::_unpack_a2a_message >>> msg_string: {:?}",
            msg_string
        );

        Ok((msg_string, sender_vk))
    }

    pub async fn anon_unpack(wallet: Arc<dyn BaseAgencyClientWallet>, payload: Vec<u8>) -> AgencyClientResult<String> {
        trace!(
            "EncryptionEnvelope::anon_unpack >>> processing payload of {} bytes",
            payload.len()
        );
        if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::anon_unpack >>> returning decrypted mock message");
            Ok(AgencyMockDecrypted::get_next_decrypted_message())
        } else {
            let (a2a_message, _sender_vk) = Self::_unpack_a2a_message(wallet, payload).await?;
            trace!("EncryptionEnvelope::anon_unpack >>> a2a_message: {:?}", a2a_message);
            Ok(a2a_message)
        }
    }

    pub async fn auth_unpack(
        wallet: Arc<dyn BaseAgencyClientWallet>,
        payload: Vec<u8>,
        expected_vk: &str,
    ) -> AgencyClientResult<String> {
        trace!(
            "EncryptionEnvelope::auth_unpack >>> processing payload of {} bytes, expected_vk={}",
            payload.len(),
            expected_vk
        );

        if AgencyMockDecrypted::has_decrypted_mock_messages() {
            trace!("EncryptionEnvelope::auth_unpack >>> returning decrypted mock message");
            Ok(AgencyMockDecrypted::get_next_decrypted_message())
        } else {
            let (a2a_message, sender_vk) = Self::_unpack_a2a_message(wallet, payload).await?;
            trace!(
                "EncryptionEnvelope::auth_unpack >>> a2a_message: {:?}, sender_vk: {:?}",
                a2a_message,
                sender_vk
            );

            match sender_vk {
                Some(sender_vk) => {
                    if sender_vk != expected_vk {
                        error!(
                            "auth_unpack :: sender_vk != expected_vk.... sender_vk={}, expected_vk={}",
                            sender_vk, expected_vk
                        );
                        return Err(AgencyClientError::from_msg(
                            AgencyClientErrorKind::InvalidJson,
                            format!(
                                "Message did not pass authentication check. Expected sender verkey was {}, but \
                                 actually was {}",
                                expected_vk, sender_vk
                            ),
                        ));
                    }
                }
                None => {
                    error!("auth_unpack :: message was authcrypted");
                    return Err(AgencyClientError::from_msg(
                        AgencyClientErrorKind::InvalidJson,
                        "Can't authenticate message because it was anoncrypted.",
                    ));
                }
            }
            trace!("EncryptionEnvelope::auth_unpack >>> a2a_message: {:?}", a2a_message);
            Ok(a2a_message)
        }
    }
}
