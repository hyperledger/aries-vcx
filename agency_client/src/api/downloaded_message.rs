use std::sync::Arc;

use crate::{
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    utils::encryption_envelope::EncryptionEnvelope,
    wallet::base_agency_client_wallet::BaseAgencyClientWallet,
    MessageStatusCode,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum MessagePayload {
    V2(::serde_json::Value),
}

impl Default for MessagePayload {
    fn default() -> Self {
        Self::V2(::serde_json::Value::Null)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedMessageEncrypted {
    pub status_code: MessageStatusCode,
    pub payload: MessagePayload,
    pub uid: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedMessage {
    pub status_code: MessageStatusCode,
    pub uid: String,
    pub decrypted_msg: String,
}

impl DownloadedMessageEncrypted {
    pub fn payload(&self) -> AgencyClientResult<Vec<u8>> {
        match &self.payload {
            MessagePayload::V2(payload) => serde_json::to_vec(payload)
                .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, err)),
        }
    }

    pub async fn decrypt_noauth(
        self,
        wallet: Arc<dyn BaseAgencyClientWallet>,
    ) -> AgencyClientResult<DownloadedMessage> {
        let decrypted_payload = self._noauth_decrypt_v3_message(wallet).await?;
        Ok(DownloadedMessage {
            status_code: self.status_code.clone(),
            uid: self.uid.clone(),
            decrypted_msg: decrypted_payload,
        })
    }

    pub async fn decrypt_auth(
        self,
        wallet: Arc<dyn BaseAgencyClientWallet>,
        expected_sender_vk: &str,
    ) -> AgencyClientResult<DownloadedMessage> {
        let decrypted_payload = self._auth_decrypt_v3_message(wallet, expected_sender_vk).await?;
        Ok(DownloadedMessage {
            status_code: self.status_code.clone(),
            uid: self.uid.clone(),
            decrypted_msg: decrypted_payload,
        })
    }

    async fn _noauth_decrypt_v3_message(&self, wallet: Arc<dyn BaseAgencyClientWallet>) -> AgencyClientResult<String> {
        EncryptionEnvelope::anon_unpack(wallet, self.payload()?).await
    }

    async fn _auth_decrypt_v3_message(
        &self,
        wallet: Arc<dyn BaseAgencyClientWallet>,
        expected_sender_vk: &str,
    ) -> AgencyClientResult<String> {
        EncryptionEnvelope::auth_unpack(wallet, self.payload()?, expected_sender_vk).await
    }
}
