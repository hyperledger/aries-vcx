use aries::utils::encryption_envelope::EncryptionEnvelope;
use error::{VcxError, VcxErrorKind, VcxResult};
use messages::{A2AMessage, A2AMessageKinds, A2AMessageV2, GeneralMessage, get_messages, MessageStatusCode, parse_response_from_agency, prepare_message_for_agency, prepare_message_for_agent, RemoteMessageType};
use messages::message_type::MessageTypes;
use settings;
use settings::ProtocolTypes;
use utils::{constants, httpclient};
use utils::httpclient::AgencyMock;

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
    version: ProtocolTypes,
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
            version: settings::get_protocol_type(),
        }
    }

    #[cfg(test)]
    pub fn create_v1() -> GetMessagesBuilder {
        let mut builder = GetMessagesBuilder::create();
        builder.version = settings::ProtocolTypes::V1;
        builder
    }

    pub fn uid(&mut self, uids: Option<Vec<String>>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uids = uids;
        Ok(self)
    }

    pub fn status_codes(&mut self, status_codes: Option<Vec<MessageStatusCode>>) -> VcxResult<&mut Self> {
        self.status_codes = status_codes;
        Ok(self)
    }

    pub fn pairwise_dids(&mut self, pairwise_dids: Option<Vec<String>>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.pairwise_dids = pairwise_dids;
        Ok(self)
    }

    pub fn include_edge_payload(&mut self, payload: &str) -> VcxResult<&mut Self> {
        //todo: is this a json value, String??
        self.exclude_payload = Some(payload.to_string());
        Ok(self)
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<Vec<Message>> {
        debug!("GetMessages::send >>> self.agent_vk={} self.agent_did={} self.to_did={} self.to_vk={}", self.agent_vk, self.agent_did, self.to_did, self.to_vk);

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(response)
    }

    fn parse_response(&self, response: Vec<u8>) -> VcxResult<Vec<Message>> {
        trace!("parse_get_messages_response >>> response: {:?}", response);

        let mut response = parse_response_from_agency(&response, &self.version)?;

        trace!("parse_get_messages_response >>> obtained agency response {:?}", response);

        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::GetMessagesResponse(res)) => {
                trace!("Interpreting response as V2");
                Ok(res.msgs)
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesResponse"))
        }
    }

    pub fn download_messages_noauth(&mut self) -> VcxResult<Vec<MessageByConnection>> {
        trace!("GetMessages::download >>>");

        let data = self.prepare_download_request()?;

        let response = httpclient::post_u8(&data)?;

        if settings::agency_mocks_enabled() && response.len() == 0 {
            return Ok(Vec::new());
        }

        let response = self.parse_download_messages_response(response)?;

        Ok(response)
    }

    fn prepare_download_request(&self) -> VcxResult<Vec<u8>> {
        let message = A2AMessage::Version2(
            A2AMessageV2::GetMessages(
                GetMessages::build(A2AMessageKinds::GetMessagesByConnections,
                                   self.exclude_payload.clone(),
                                   self.uids.clone(),
                                   self.status_codes.clone(),
                                   self.pairwise_dids.clone()))
        );

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_download_messages_response(&self, response: Vec<u8>) -> VcxResult<Vec<MessageByConnection>> {
        trace!("parse_download_messages_response >>>");
        let mut response = parse_response_from_agency(&response, &self.version)?;

        trace!("parse_download_messages_response: parsed response {:?}", response);
        let msgs = match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::GetMessagesByConnectionsResponse(res)) => res.msgs,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesByConnectionsResponse"))
        };

        msgs
            .iter()
            .map(|connection| {
                MessageByConnection {
                    pairwise_did: connection.pairwise_did.clone(),
                    msgs: connection.msgs.iter().map(|message| message.decrypt()).collect(),
                }
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

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        debug!("prepare_request >> This connection is using protocol_type: {:?}", self.version);
        let message = match self.version {
            settings::ProtocolTypes::V1 |
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 |
            settings::ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::GetMessages(
                        GetMessages::build(A2AMessageKinds::GetMessages,
                                           self.exclude_payload.clone(),
                                           self.uids.clone(),
                                           self.status_codes.clone(),
                                           self.pairwise_dids.clone()))
                ),
        };

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk, &self.version)
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
    pub fn payload(&self) -> VcxResult<Vec<u8>> {
        match self.payload {
            Some(MessagePayload::V2(ref payload)) => serde_json::to_vec(payload).map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, err)),
            _ => Err(VcxError::from(VcxErrorKind::InvalidState)),
        }
    }

    // todo: must use vk to verify send of the message
    pub fn decrypt(&self) -> Message {
        // TODO: must be Result
        let mut new_message = self.clone();
        if let Ok(decrypted_msg) = self._decrypt_v3_message() {
            new_message.decrypted_msg = Some(decrypted_msg);
        } else {
            new_message.decrypted_msg = None;
        }
        new_message.payload = None;
        new_message
    }

    fn _decrypt_v3_message(&self) -> VcxResult<String> {
        use aries::utils::encryption_envelope::EncryptionEnvelope;
        let a2a_message = EncryptionEnvelope::anon_unpack(self.payload()?)?;
        Ok(json!(&a2a_message).to_string())
    }
}

pub fn get_connection_messages(pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>, version: &Option<ProtocolTypes>) -> VcxResult<Vec<Message>> {
    trace!("get_connection_messages >>> pw_did: {}, pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           pw_did, pw_vk, agent_vk, msg_uid);

    let response = get_messages()
        .to(&pw_did)?
        .to_vk(&pw_vk)?
        .agent_did(&agent_did)?
        .agent_vk(&agent_vk)?
        .uid(msg_uid)?
        .status_codes(status_codes)?
        .version(version)?
        .send_secure()
        .map_err(|err| err.map(VcxErrorKind::PostMessageFailed, "Cannot get messages"))?;

    trace!("message returned: {:?}", response);
    Ok(response)
}

fn _parse_status_code(status_codes: Option<Vec<String>>) -> VcxResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code|
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code))
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse message status code: {}", err)))
                ).collect::<VcxResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None)
    }
}

pub fn download_messages_noauth(pairwise_dids: Option<Vec<String>>, status_codes: Option<Vec<String>>, uids: Option<Vec<String>>) -> VcxResult<Vec<MessageByConnection>> {
    trace!("download_messages_noauth >>> pairwise_dids: {:?}, status_codes: {:?}, uids: {:?}",
           pairwise_dids, status_codes, uids);

    let status_codes = _parse_status_code(status_codes)?;

    let response =
        get_messages()
            .uid(uids)?
            .status_codes(status_codes)?
            .pairwise_dids(pairwise_dids)?
            .version(&Some(::settings::get_protocol_type()))?
            .download_messages_noauth()?;

    trace!("message returned: {:?}", response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "agency_pool_tests")]
    use std::thread;
    #[cfg(feature = "agency_pool_tests")]
    use std::time::Duration;

    use connection::send_generic_message;
    use utils::constants::{GET_ALL_MESSAGES_RESPONSE, GET_MESSAGES_RESPONSE};
    use utils::devsetup::*;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    #[cfg(feature = "to_restore")]
    fn test_parse_get_messages_response() {
        let _setup = SetupAriesMocks::init();

        // we should setup keys, build encrypted response from agency
        // then test we are able to decrypt th message

        // old test:
        // let result = GetMessagesBuilder::create_v1().parse_response(GET_MESSAGES_RESPONSE.to_vec()).unwrap();
        // assert_eq!(result.len(), 3)
    }

    #[test]
    #[cfg(feature = "general_test")]
    #[cfg(feature = "to_restore")]
    fn test_parse_get_connection_messages_response() {
        let _setup = SetupAriesMocks::init();

        // we should setup keys, build encrypted response from agency
        // then test we are able to decrypt th message

        // old test:
        // let result = GetMessagesBuilder::create().version(&Some(ProtocolTypes::V1)).unwrap().parse_download_messages_response(GET_ALL_MESSAGES_RESPONSE.to_vec()).unwrap();
        // assert_eq!(result.len(), 1)
    }
}
