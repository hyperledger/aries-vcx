use futures::StreamExt;

use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageType;
use crate::messages::a2a_message::A2AMessageKinds;
use crate::MessageStatusCode;
use crate::utils::encryption_envelope::EncryptionEnvelope;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessages {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "excludePayload")]
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uids: Option<Vec<String>>,
    #[serde(rename = "statusCodes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    status_codes: Option<Vec<MessageStatusCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pairwiseDIDs")]
    pairwise_dids: Option<Vec<String>>,
}

impl GetMessages {
    pub(crate) fn build(kind: A2AMessageKinds, exclude_payload: Option<String>, uids: Option<Vec<String>>,
                        status_codes: Option<Vec<MessageStatusCode>>, pairwise_dids: Option<Vec<String>>) -> GetMessages {
        GetMessages {
            msg_type: MessageType::build_v2(kind),
            exclude_payload,
            uids,
            status_codes,
            pairwise_dids,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesResponse {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    pub msgs: Vec<DownloadedMessageEncrypted>,
}

#[derive(Debug)]
pub struct GetMessagesBuilder {
    exclude_payload: Option<String>,
    uids: Option<Vec<String>>,
    status_codes: Option<Vec<MessageStatusCode>>,
    pairwise_dids: Option<Vec<String>>,
}

impl GetMessagesBuilder {
    pub fn create() -> GetMessagesBuilder {
        trace!("GetMessages::create_message >>>");

        GetMessagesBuilder {
            uids: None,
            exclude_payload: None,
            status_codes: None,
            pairwise_dids: None,
        }
    }

    pub fn uid(&mut self, uids: Option<Vec<String>>) -> AgencyClientResult<&mut Self> {
        self.uids = uids;
        Ok(self)
    }

    pub fn status_codes(&mut self, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<&mut Self> {
        self.status_codes = status_codes;
        Ok(self)
    }

    pub fn build(&self) -> GetMessages {
        GetMessages::build(A2AMessageKinds::GetMessages,
                           self.exclude_payload.clone(),
                           self.uids.clone(),
                           self.status_codes.clone(),
                           self.pairwise_dids.clone())
    }
}

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

    pub async fn decrypt_noauth(self) -> AgencyClientResult<DownloadedMessage> {
        let decrypted_payload = self._noauth_decrypt_v3_message().await?;
        Ok(DownloadedMessage {
            status_code: self.status_code.clone(),
            uid: self.uid.clone(),
            decrypted_msg: decrypted_payload,
        })
    }

    pub async fn decrypt_auth(self, expected_sender_vk: &str) -> AgencyClientResult<DownloadedMessage> {
        let decrypted_payload = self._auth_decrypt_v3_message(expected_sender_vk).await?;
        Ok(DownloadedMessage {
            status_code: self.status_code.clone(),
            uid: self.uid.clone(),
            decrypted_msg: decrypted_payload,
        })
    }

    async fn _noauth_decrypt_v3_message(&self) -> AgencyClientResult<String> {
        EncryptionEnvelope::anon_unpack(self.payload()?).await
    }

    async fn _auth_decrypt_v3_message(&self, expected_sender_vk: &str) -> AgencyClientResult<String> {
        EncryptionEnvelope::auth_unpack(self.payload()?, &expected_sender_vk).await
    }
}
