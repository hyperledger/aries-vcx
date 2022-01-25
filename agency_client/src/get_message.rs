use crate::{A2AMessage, A2AMessageKinds, A2AMessageV2, agency_settings, GeneralMessage, get_messages, MessageStatusCode, mocking, parse_response_from_agency, prepare_message_for_agency, prepare_message_for_agent};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesByConnections {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "msgsByConns")]
    #[serde(default)]
    msgs: Vec<MessageByConnection>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageByConnection {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub msgs: Vec<Message>,
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

    #[cfg(test)]
    pub fn create_v1() -> GetMessagesBuilder {
        GetMessagesBuilder::create()
    }

    pub fn uid(&mut self, uids: Option<Vec<String>>) -> AgencyClientResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uids = uids;
        Ok(self)
    }

    pub fn status_codes(&mut self, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<&mut Self> {
        self.status_codes = status_codes;
        Ok(self)
    }

    pub fn pairwise_dids(&mut self, pairwise_dids: Option<Vec<String>>) -> AgencyClientResult<&mut Self> {
        //Todo: validate msg_uid??
        self.pairwise_dids = pairwise_dids;
        Ok(self)
    }

    pub fn include_edge_payload(&mut self, payload: &str) -> AgencyClientResult<&mut Self> {
        //todo: is this a json value, String??
        self.exclude_payload = Some(payload.to_string());
        Ok(self)
    }

    pub async fn send_secure(&mut self) -> AgencyClientResult<Vec<Message>> {
        debug!("GetMessages::send >>> self.agent_vk={} self.agent_did={} self.to_did={} self.to_vk={}", self.agent_vk, self.agent_did, self.to_did, self.to_vk);

        let data = self.prepare_request()?;

        let response = post_to_agency(&data).await?;

        self.parse_response(response)
    }

    fn parse_response(&self, response: Vec<u8>) -> AgencyClientResult<Vec<Message>> {
        trace!("parse_get_messages_response >>> processing payload of {} bytes", response.len());

        let mut response = parse_response_from_agency(&response)?;

        trace!("parse_get_messages_response >>> obtained agency response {:?}", response);

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::GetMessagesResponse(res)) => {
                trace!("Interpreting response as V2");
                Ok(res.msgs)
            }
            _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesResponse"))
        }
    }

    pub async fn download_messages_noauth(&mut self) -> AgencyClientResult<Vec<MessageByConnection>> {
        trace!("GetMessages::download >>>");

        let data = self.prepare_download_request()?;

        let response = post_to_agency(&data).await?;

        if mocking::agency_mocks_enabled() && response.len() == 0 {
            return Ok(Vec::new());
        }

        let response = self.parse_download_messages_response_noauth(response)?;

        Ok(response)
    }

    fn prepare_download_request(&self) -> AgencyClientResult<Vec<u8>> {
        let message = A2AMessage::Version2(
            A2AMessageV2::GetMessages(
                GetMessages::build(A2AMessageKinds::GetMessagesByConnections,
                                   self.exclude_payload.clone(),
                                   self.uids.clone(),
                                   self.status_codes.clone(),
                                   self.pairwise_dids.clone()))
        );

        let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did)
    }

    // todo: This should be removed after public method vcx_messages_download is removed
    fn parse_download_messages_response_noauth(&self, response: Vec<u8>) -> AgencyClientResult<Vec<MessageByConnection>> {
        trace!("parse_download_messages_response >>>");
        let mut response = parse_response_from_agency(&response)?;

        trace!("parse_download_messages_response: parsed response {:?}", response);
        let msgs = match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::GetMessagesByConnectionsResponse(res)) => res.msgs,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesByConnectionsResponse"))
        };

        msgs
            .iter()
            .map(|connection| {
                Ok(MessageByConnection {
                    pairwise_did: connection.pairwise_did.clone(),
                    msgs: connection.msgs.iter().map(|message| message.decrypt_noauth()).collect(),
                })
            })
            .collect()
    }
}

//Todo: Every GeneralMessage extension, duplicates code
impl GeneralMessage for GetMessagesBuilder {
    type Msg = GetMessagesBuilder;

    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }
    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_agent_did(&mut self, did: String) { self.agent_did = did; }
    fn set_agent_vk(&mut self, vk: String) { self.agent_vk = vk; }

    fn prepare_request(&mut self) -> AgencyClientResult<Vec<u8>> {
        debug!("prepare_request >>");
        let message = A2AMessage::Version2(
            A2AMessageV2::GetMessages(
                GetMessages::build(A2AMessageKinds::GetMessages,
                                   self.exclude_payload.clone(),
                                   self.uids.clone(),
                                   self.status_codes.clone(),
                                   self.pairwise_dids.clone()))
        );

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDetails {
    to: String,
    status_code: String,
    last_updated_date_time: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum MessagePayload {
    V2(::serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "statusCode")]
    pub status_code: MessageStatusCode,
    pub payload: Option<MessagePayload>,
    pub uid: String,
    pub ref_msg_id: Option<String>,
    #[serde(skip_deserializing)]
    pub delivery_details: Vec<DeliveryDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decrypted_msg: Option<String>,
}

#[macro_export]
macro_rules! convert_aries_message {
    ($a2a_msg:ident, $kind:ident) => (
         (PayloadKinds::$kind, json!(&$a2a_msg).to_string())
    )
}

impl Message {
    pub fn payload(&self) -> AgencyClientResult<Vec<u8>> {
        match self.payload {
            Some(MessagePayload::V2(ref payload)) => serde_json::to_vec(payload).map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, err)),
            _ => Err(AgencyClientError::from(AgencyClientErrorKind::InvalidState)),
        }
    }

    pub fn decrypt_noauth(&self) -> Message {
        let mut new_message = self.clone();
        if let Ok(decrypted_msg) = self._noauth_decrypt_v3_message() {
            new_message.decrypted_msg = Some(decrypted_msg);
        } else {
            new_message.decrypted_msg = None;
        }
        new_message.payload = None;
        new_message
    }

    pub fn decrypt_auth(&self, expected_sender_vk: &str) -> AgencyClientResult<Message> {
        let mut new_message = self.clone();
        let decrypted_msg = self._auth_decrypt_v3_message(expected_sender_vk)?;
        trace!("decrypt_auth >>> decrypted_msg: {:?}", decrypted_msg);
        new_message.decrypted_msg = Some(decrypted_msg);
        new_message.payload = None;
        Ok(new_message)
    }

    fn _noauth_decrypt_v3_message(&self) -> AgencyClientResult<String> {
        EncryptionEnvelope::anon_unpack(self.payload()?)
    }

    fn _auth_decrypt_v3_message(&self, expected_sender_vk: &str) -> AgencyClientResult<String> {
        EncryptionEnvelope::auth_unpack(self.payload()?, &expected_sender_vk)
    }
}

pub async fn get_connection_messages(pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>) -> AgencyClientResult<Vec<Message>> {
    trace!("get_connection_messages >>> pw_did: {}, pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           pw_did, pw_vk, agent_vk, msg_uid);

    let response = get_messages()
        .to(&pw_did)?
        .to_vk(&pw_vk)?
        .agent_did(&agent_did)?
        .agent_vk(&agent_vk)?
        .uid(msg_uid)?
        .status_codes(status_codes)?
        .send_secure()
        .await
        .map_err(|err| err.map(AgencyClientErrorKind::PostMessageFailed, "Cannot get messages"))?;

    trace!("message returned: {:?}", response);
    Ok(response)
}

pub fn parse_status_codes(status_codes: Option<Vec<String>>) -> AgencyClientResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code|
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code))
                        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot parse message status code: {}", err)))
                ).collect::<AgencyClientResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None)
    }
}

pub fn parse_connection_handles(conn_handles: Vec<String>) -> AgencyClientResult<Vec<u32>> {
    trace!("parse_connection_handles >>> conn_handles: {:?}", conn_handles);
    let codes = conn_handles
        .iter()
        .map(|handle|
            ::serde_json::from_str::<u32>(handle)
                .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot parse connection handles: {}", err)))
        ).collect::<AgencyClientResult<Vec<u32>>>()?;
    Ok(codes)
}

pub async fn download_messages_noauth(pairwise_dids: Option<Vec<String>>, status_codes: Option<Vec<String>>, uids: Option<Vec<String>>) -> AgencyClientResult<Vec<MessageByConnection>> {
    trace!("download_messages_noauth >>> pairwise_dids: {:?}, status_codes: {:?}, uids: {:?}",
           pairwise_dids, status_codes, uids);

    let status_codes = parse_status_codes(status_codes)?;

    let response =
        get_messages()
            .uid(uids)?
            .status_codes(status_codes)?
            .pairwise_dids(pairwise_dids)?
            .download_messages_noauth()
            .await?;

    trace!("message returned: {:?}", response);
    Ok(response)
}
