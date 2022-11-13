use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::utils::encryption_envelope::EncryptionEnvelope;
use crate::MessageStatusCode;
use vdrtools::WalletHandle;

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

    pub async fn decrypt_noauth(self, wallet_handle: WalletHandle) -> AgencyClientResult<DownloadedMessage> {
        let decrypted_payload = self._noauth_decrypt_v3_message(wallet_handle).await?;
        Ok(DownloadedMessage {
            status_code: self.status_code.clone(),
            uid: self.uid.clone(),
            decrypted_msg: decrypted_payload,
        })
    }

    pub async fn decrypt_auth(
        self,
        wallet_handle: WalletHandle,
        expected_sender_vk: &str,
    ) -> AgencyClientResult<DownloadedMessage> {
        let decrypted_payload = self._auth_decrypt_v3_message(wallet_handle, expected_sender_vk).await?;
        Ok(DownloadedMessage {
            status_code: self.status_code.clone(),
            uid: self.uid.clone(),
            decrypted_msg: decrypted_payload,
        })
    }

    async fn _noauth_decrypt_v3_message(&self, wallet_handle: WalletHandle) -> AgencyClientResult<String> {
        EncryptionEnvelope::anon_unpack(wallet_handle, self.payload()?).await
    }

    async fn _auth_decrypt_v3_message(
        &self,
        wallet_handle: WalletHandle,
        expected_sender_vk: &str,
    ) -> AgencyClientResult<String> {
        EncryptionEnvelope::auth_unpack(wallet_handle, self.payload()?, expected_sender_vk).await
    }
}
