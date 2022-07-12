use async_trait::async_trait;
use futures::StreamExt;
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
use crate::{agency_settings, GeneralMessage, MessageStatusCode, parse_response_from_agency, prepare_message_for_agency, prepare_message_for_agent};
use crate::messages::a2a_message::{AgencyMsg, A2AMessageKinds, AgencyMessageTypes};
use crate::testing::mocking;
use crate::utils::comm::post_to_agency;
use crate::utils::encryption_envelope::EncryptionEnvelope;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessages {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
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
    fn build(kind: A2AMessageKinds, exclude_payload: Option<String>, uids: Option<Vec<String>>,
             status_codes: Option<Vec<MessageStatusCode>>, pairwise_dids: Option<Vec<String>>) -> GetMessages {
        GetMessages {
            msg_type: MessageTypes::build(kind),
            exclude_payload,
            uids,
            status_codes,
            pairwise_dids,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    msgs: Vec<Message>,
}

#[derive(Debug)]
pub struct GetMessagesBuilder {
    to_did: String,
    to_vk: String,
    agent_did: String,
    agent_vk: String,
    exclude_payload: Option<String>,
    uids: Option<Vec<String>>,
    status_codes: Option<Vec<MessageStatusCode>>,
    pairwise_dids: Option<Vec<String>>,
}

impl GetMessagesBuilder {
    pub fn create() -> GetMessagesBuilder {
        trace!("GetMessages::create_message >>>");

        GetMessagesBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
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

    pub async fn send_secure(&mut self) -> AgencyClientResult<Vec<Message>> {
        debug!("GetMessages::send >>> self.agent_vk={} self.agent_did={} self.to_did={} self.to_vk={}", self.agent_vk, self.agent_did, self.to_did, self.to_vk);

        let data = self.prepare_request().await?;

        let response = post_to_agency(&data).await?;

        self.parse_response(response).await
    }

    async fn parse_response(&self, response: Vec<u8>) -> AgencyClientResult<Vec<Message>> {
        trace!("parse_get_messages_response >>> processing payload of {} bytes", response.len());

        let mut response = parse_response_from_agency(&response).await?;

        trace!("parse_get_messages_response >>> obtained agency response {:?}", response);

        match response.remove(0) {
            AgencyMsg::Version2(AgencyMessageTypes::GetMessagesResponse(res)) => {
                trace!("Interpreting response as V2");
                Ok(res.msgs)
            }
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesResponse"))
        }
    }
}

//Todo: Every GeneralMessage extension, duplicates code
#[async_trait]
impl GeneralMessage for GetMessagesBuilder {
    type Msg = GetMessagesBuilder;

    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }
    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_agent_did(&mut self, did: String) { self.agent_did = did; }
    fn set_agent_vk(&mut self, vk: String) { self.agent_vk = vk; }

    async fn prepare_request(&mut self) -> AgencyClientResult<Vec<u8>> {
        debug!("prepare_request >>");
        let message = AgencyMsg::Version2(
            AgencyMessageTypes::GetMessages(
                GetMessages::build(A2AMessageKinds::GetMessages,
                                   self.exclude_payload.clone(),
                                   self.uids.clone(),
                                   self.status_codes.clone(),
                                   self.pairwise_dids.clone()))
        );

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk).await
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "statusCode")]
    pub status_code: MessageStatusCode,
    pub payload: Option<MessagePayload>,
    pub uid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decrypted_msg: Option<String>,
}

impl Message {
    pub fn payload(&self) -> AgencyClientResult<Vec<u8>> {
        match self.payload {
            Some(MessagePayload::V2(ref payload)) => serde_json::to_vec(payload).map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, err)),
            _ => Err(AgencyClientError::from(AgencyClientErrorKind::InvalidState)),
        }
    }

    pub async fn decrypt_noauth(&self) -> Message {
        let mut new_message = self.clone();
        if let Ok(decrypted_msg) = self._noauth_decrypt_v3_message().await {
            new_message.decrypted_msg = Some(decrypted_msg);
        } else {
            new_message.decrypted_msg = None;
        }
        new_message.payload = None;
        new_message
    }

    pub async fn decrypt_auth(self, expected_sender_vk: &str) -> AgencyClientResult<Message> {
        let mut new_message = self.clone();
        let decrypted_msg = self._auth_decrypt_v3_message(expected_sender_vk).await?;
        trace!("decrypt_auth >>> decrypted_msg: {:?}", decrypted_msg);
        new_message.decrypted_msg = Some(decrypted_msg);
        new_message.payload = None;
        Ok(new_message)
    }

    async fn _noauth_decrypt_v3_message(&self) -> AgencyClientResult<String> {
        EncryptionEnvelope::anon_unpack(self.payload()?).await
    }

    async fn _auth_decrypt_v3_message(&self, expected_sender_vk: &str) -> AgencyClientResult<String> {
        EncryptionEnvelope::auth_unpack(self.payload()?, &expected_sender_vk).await
    }
}
