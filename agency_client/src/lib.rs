#![crate_name = "agency_client"]
extern crate failure;
extern crate futures;
extern crate indyrs as indy;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate url;
extern crate async_std;
extern crate async_trait;

pub mod get_message;
mod utils;
pub mod update_connection;
pub mod update_message;
pub mod message_type;
pub mod payload;
#[macro_use]
pub mod agency_settings;
pub mod mocking;
pub mod httpclient;
pub mod agency_client;
pub mod agent_utils;
pub mod error;

use std::u8;

use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde_json::Value;

use self::error::prelude::*;
use self::utils::libindy::crypto;

use self::agent_utils::{ComMethodUpdated, Connect, ConnectResponse, CreateAgent, CreateAgentResponse, SignUp, SignUpResponse, UpdateComMethod};
use self::utils::validation;
use self::utils::create_key::{CreateKey, CreateKeyBuilder, CreateKeyResponse};
use self::get_message::{GetMessages, GetMessagesBuilder, GetMessagesResponse, MessagesByConnections};
use self::message_type::*;
use self::update_connection::{DeleteConnectionBuilder, UpdateConnection, UpdateConnectionResponse};
use self::update_message::{UpdateMessageStatusByConnections, UpdateMessageStatusByConnectionsResponse};
use self::utils::update_profile::{UpdateConfigs, UpdateConfigsResponse, UpdateProfileDataBuilder};
use self::mocking::AgencyMockDecrypted;
use async_trait::async_trait;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum A2AMessageV2 {
    /// routing
    Forward(ForwardV2),

    /// onbording
    Connect(Connect),
    ConnectResponse(ConnectResponse),
    SignUp(SignUp),
    SignUpResponse(SignUpResponse),
    CreateAgent(CreateAgent),
    CreateAgentResponse(CreateAgentResponse),

    /// PW Connection
    CreateKey(CreateKey),
    CreateKeyResponse(CreateKeyResponse),

    SendRemoteMessage(SendRemoteMessage),
    SendRemoteMessageResponse(SendRemoteMessageResponse),

    GetMessages(GetMessages),
    GetMessagesResponse(GetMessagesResponse),
    GetMessagesByConnections(GetMessages),
    GetMessagesByConnectionsResponse(MessagesByConnections),

    UpdateConnection(UpdateConnection),
    UpdateConnectionResponse(UpdateConnectionResponse),
    UpdateMessageStatusByConnections(UpdateMessageStatusByConnections),
    UpdateMessageStatusByConnectionsResponse(UpdateMessageStatusByConnectionsResponse),

    /// config
    UpdateConfigs(UpdateConfigs),
    UpdateConfigsResponse(UpdateConfigsResponse),
    UpdateComMethod(UpdateComMethod),
    ComMethodUpdated(ComMethodUpdated),
}

impl<'de> Deserialize<'de> for A2AMessageV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        let message_type: MessageType = serde_json::from_value(value["@type"].clone()).map_err(de::Error::custom)?;

        if log::log_enabled!(log::Level::Trace) {
            let message_json = serde_json::ser::to_string(&value);
            let message_type_json = serde_json::ser::to_string(&value["@type"].clone());

            trace!("Deserializing A2AMessageV2 json: {:?}", &message_json);
            trace!("Found A2AMessageV2 message type json {:?}", &message_type_json);
            trace!("Found A2AMessageV2 message type {:?}", &message_type);
        };

        match message_type.type_.as_str() {
            "FWD" => {
                ForwardV2::deserialize(value)
                    .map(A2AMessageV2::Forward)
                    .map_err(de::Error::custom)
            }
            "CONNECT" => {
                Connect::deserialize(value)
                    .map(A2AMessageV2::Connect)
                    .map_err(de::Error::custom)
            }
            "CONNECTED" => {
                ConnectResponse::deserialize(value)
                    .map(A2AMessageV2::ConnectResponse)
                    .map_err(de::Error::custom)
            }
            "SIGNUP" => {
                SignUp::deserialize(value)
                    .map(A2AMessageV2::SignUp)
                    .map_err(de::Error::custom)
            }
            "SIGNED_UP" => {
                SignUpResponse::deserialize(value)
                    .map(A2AMessageV2::SignUpResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_AGENT" => {
                CreateAgent::deserialize(value)
                    .map(A2AMessageV2::CreateAgent)
                    .map_err(de::Error::custom)
            }
            "AGENT_CREATED" => {
                CreateAgentResponse::deserialize(value)
                    .map(A2AMessageV2::CreateAgentResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_KEY" => {
                CreateKey::deserialize(value)
                    .map(A2AMessageV2::CreateKey)
                    .map_err(de::Error::custom)
            }
            "KEY_CREATED" => {
                CreateKeyResponse::deserialize(value)
                    .map(A2AMessageV2::CreateKeyResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS" => {
                GetMessages::deserialize(value)
                    .map(A2AMessageV2::GetMessages)
                    .map_err(de::Error::custom)
            }
            "MSGS" => {
                GetMessagesResponse::deserialize(value)
                    .map(A2AMessageV2::GetMessagesResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS_BY_CONNS" => {
                GetMessages::deserialize(value)
                    .map(A2AMessageV2::GetMessagesByConnections)
                    .map_err(de::Error::custom)
            }
            "MSGS_BY_CONNS" => {
                MessagesByConnections::deserialize(value)
                    .map(A2AMessageV2::GetMessagesByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "SEND_REMOTE_MSG" => {
                SendRemoteMessage::deserialize(value)
                    .map(A2AMessageV2::SendRemoteMessage)
                    .map_err(de::Error::custom)
            }
            "REMOTE_MSG_SENT" => {
                SendRemoteMessageResponse::deserialize(value)
                    .map(A2AMessageV2::SendRemoteMessageResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONN_STATUS" => {
                UpdateConnection::deserialize(value)
                    .map(A2AMessageV2::UpdateConnection)
                    .map_err(de::Error::custom)
            }
            "CONN_STATUS_UPDATED" => {
                UpdateConnectionResponse::deserialize(value)
                    .map(A2AMessageV2::UpdateConnectionResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_MSG_STATUS_BY_CONNS" => {
                UpdateMessageStatusByConnections::deserialize(value)
                    .map(A2AMessageV2::UpdateMessageStatusByConnections)
                    .map_err(de::Error::custom)
            }
            "MSG_STATUS_UPDATED_BY_CONNS" => {
                UpdateMessageStatusByConnectionsResponse::deserialize(value)
                    .map(A2AMessageV2::UpdateMessageStatusByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONFIGS" => {
                UpdateConfigs::deserialize(value)
                    .map(A2AMessageV2::UpdateConfigs)
                    .map_err(de::Error::custom)
            }
            "CONFIGS_UPDATED" => {
                UpdateConfigsResponse::deserialize(value)
                    .map(A2AMessageV2::UpdateConfigsResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_COM_METHOD" => {
                UpdateComMethod::deserialize(value)
                    .map(A2AMessageV2::UpdateComMethod)
                    .map_err(de::Error::custom)
            }
            "COM_METHOD_UPDATED" => {
                ComMethodUpdated::deserialize(value)
                    .map(A2AMessageV2::ComMethodUpdated)
                    .map_err(de::Error::custom)
            }
            _ => Err(de::Error::custom("Unexpected @type field structure."))
        }
    }
}

// We don't want to use this anymore
#[derive(Debug)]
pub enum A2AMessage {
    Version2(A2AMessageV2),
}

impl Serialize for A2AMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            A2AMessage::Version2(msg) => msg.serialize(serializer).map_err(ser::Error::custom)
        }
    }
}

impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        let message_type: MessageTypes = serde_json::from_value(value["@type"].clone()).map_err(de::Error::custom)?;

        if log::log_enabled!(log::Level::Trace) {
            let message_json = serde_json::ser::to_string(&value);
            let message_type_json = serde_json::ser::to_string(&value["@type"].clone());

            trace!("Deserializing A2AMessage json: {:?}", &message_json);
            trace!("Found A2AMessage message type json {:?}", &message_type_json);
            trace!("Found A2AMessage message type {:?}", &message_type);
        }

        match message_type {
            MessageTypes::MessageType(_) =>
                A2AMessageV2::deserialize(value)
                    .map(A2AMessage::Version2)
                    .map_err(de::Error::custom)
        }
    }
}


#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ForwardV2 {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "@fwd")]
    fwd: String,
    #[serde(rename = "@msg")]
    msg: Value,
}

impl ForwardV2 {
    fn new(fwd: String, msg: Vec<u8>) -> AgencyClientResult<A2AMessage> {
        let msg = serde_json::from_slice(msg.as_slice())
            .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidState, err))?;

        Ok(A2AMessage::Version2(A2AMessageV2::Forward(
            ForwardV2 {
                msg_type: MessageTypes::build_v2(A2AMessageKinds::Forward),
                fwd,
                msg,
            }
        )))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendRemoteMessage {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    #[serde(rename = "@id")]
    pub id: String,
    pub mtype: RemoteMessageType,
    #[serde(rename = "replyToMsgId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_msg_id: Option<String>,
    #[serde(rename = "sendMsg")]
    pub send_msg: bool,
    #[serde(rename = "@msg")]
    msg: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendRemoteMessageResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "@id")]
    pub id: String,
    pub sent: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RemoteMessageType {
    Other(String),
    ConnReq,
    ConnReqAnswer,
    ConnReqRedirect,
    CredOffer,
    CredReq,
    Cred,
    ProofReq,
    Proof,
}

impl Serialize for RemoteMessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            RemoteMessageType::ConnReq => "connReq",
            RemoteMessageType::ConnReqAnswer => "connReqAnswer",
            RemoteMessageType::ConnReqRedirect => "connReqRedirect",
            RemoteMessageType::CredOffer => "credOffer",
            RemoteMessageType::CredReq => "credReq",
            RemoteMessageType::Cred => "cred",
            RemoteMessageType::ProofReq => "proofReq",
            RemoteMessageType::Proof => "proof",
            RemoteMessageType::Other(_type) => _type,
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RemoteMessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("connReq") => Ok(RemoteMessageType::ConnReq),
            Some("connReqAnswer") | Some("CONN_REQ_ACCEPTED") => Ok(RemoteMessageType::ConnReqAnswer),
            Some("connReqRedirect") | Some("CONN_REQ_REDIRECTED") | Some("connReqRedirected") => Ok(RemoteMessageType::ConnReqRedirect),
            Some("credOffer") => Ok(RemoteMessageType::CredOffer),
            Some("credReq") => Ok(RemoteMessageType::CredReq),
            Some("cred") => Ok(RemoteMessageType::Cred),
            Some("proofReq") => Ok(RemoteMessageType::ProofReq),
            Some("proof") => Ok(RemoteMessageType::Proof),
            Some(_type) => Ok(RemoteMessageType::Other(_type.to_string())),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageStatusCode {
    Created,
    Sent,
    Received,
    Accepted,
    Rejected,
    Reviewed,
    Redirected,
}

impl Default for MessageStatusCode {
    fn default() -> Self {
        Self::Created
    }
}

impl MessageStatusCode {
    pub fn message(&self) -> &'static str {
        match self {
            MessageStatusCode::Created => "message created",
            MessageStatusCode::Sent => "message sent",
            MessageStatusCode::Received => "message received",
            MessageStatusCode::Redirected => "message redirected",
            MessageStatusCode::Accepted => "message accepted",
            MessageStatusCode::Rejected => "message rejected",
            MessageStatusCode::Reviewed => "message reviewed",
        }
    }
}

impl std::string::ToString for MessageStatusCode {
    fn to_string(&self) -> String {
        match self {
            MessageStatusCode::Created => "MS-101",
            MessageStatusCode::Sent => "MS-102",
            MessageStatusCode::Received => "MS-103",
            MessageStatusCode::Accepted => "MS-104",
            MessageStatusCode::Rejected => "MS-105",
            MessageStatusCode::Reviewed => "MS-106",
            MessageStatusCode::Redirected => "MS-107",
        }.to_string()
    }
}

impl Serialize for MessageStatusCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = self.to_string();
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MessageStatusCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("MS-101") => Ok(MessageStatusCode::Created),
            Some("MS-102") => Ok(MessageStatusCode::Sent),
            Some("MS-103") => Ok(MessageStatusCode::Received),
            Some("MS-104") => Ok(MessageStatusCode::Accepted),
            Some("MS-105") => Ok(MessageStatusCode::Rejected),
            Some("MS-106") => Ok(MessageStatusCode::Reviewed),
            Some("MS-107") => Ok(MessageStatusCode::Redirected),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum A2AMessageKinds {
    Forward,
    Connect,
    Connected,
    SignUp,
    SignedUp,
    CreateAgent,
    AgentCreated,
    CreateKey,
    KeyCreated,
    CreateMessage,
    MessageDetail,
    MessageCreated,
    MessageSent,
    GetMessages,
    GetMessagesByConnections,
    Messages,
    UpdateMessageStatusByConnections,
    MessageStatusUpdatedByConnections,
    UpdateConnectionStatus,
    UpdateConfigs,
    ConfigsUpdated,
    UpdateComMethod,
    ComMethodUpdated,
    SendRemoteMessage,
    SendRemoteMessageResponse,
}

impl A2AMessageKinds {
    pub fn family(&self) -> MessageFamilies {
        match self {
            A2AMessageKinds::Forward => MessageFamilies::Routing,
            A2AMessageKinds::Connect => MessageFamilies::Onboarding,
            A2AMessageKinds::Connected => MessageFamilies::Onboarding,
            A2AMessageKinds::CreateAgent => MessageFamilies::Onboarding,
            A2AMessageKinds::AgentCreated => MessageFamilies::Onboarding,
            A2AMessageKinds::SignUp => MessageFamilies::Onboarding,
            A2AMessageKinds::SignedUp => MessageFamilies::Onboarding,
            A2AMessageKinds::CreateKey => MessageFamilies::Pairwise,
            A2AMessageKinds::KeyCreated => MessageFamilies::Pairwise,
            A2AMessageKinds::CreateMessage => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageDetail => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageCreated => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageSent => MessageFamilies::Pairwise,
            A2AMessageKinds::GetMessages => MessageFamilies::Pairwise,
            A2AMessageKinds::GetMessagesByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::Messages => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateConnectionStatus => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateMessageStatusByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageStatusUpdatedByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateConfigs => MessageFamilies::Configs,
            A2AMessageKinds::ConfigsUpdated => MessageFamilies::Configs,
            A2AMessageKinds::UpdateComMethod => MessageFamilies::Configs,
            A2AMessageKinds::ComMethodUpdated => MessageFamilies::Configs,
            A2AMessageKinds::SendRemoteMessage => MessageFamilies::Routing,
            A2AMessageKinds::SendRemoteMessageResponse => MessageFamilies::Routing,
        }
    }

    pub fn name(&self) -> String {
        match self {
            A2AMessageKinds::Forward => "FWD".to_string(),
            A2AMessageKinds::Connect => "CONNECT".to_string(),
            A2AMessageKinds::Connected => "CONNECTED".to_string(),
            A2AMessageKinds::CreateAgent => "CREATE_AGENT".to_string(),
            A2AMessageKinds::AgentCreated => "AGENT_CREATED".to_string(),
            A2AMessageKinds::SignUp => "SIGNUP".to_string(),
            A2AMessageKinds::SignedUp => "SIGNED_UP".to_string(),
            A2AMessageKinds::CreateKey => "CREATE_KEY".to_string(),
            A2AMessageKinds::KeyCreated => "KEY_CREATED".to_string(),
            A2AMessageKinds::CreateMessage => "CREATE_MSG".to_string(),
            A2AMessageKinds::MessageDetail => "MSG_DETAIL".to_string(),
            A2AMessageKinds::MessageCreated => "MSG_CREATED".to_string(),
            A2AMessageKinds::MessageSent => "MSGS_SENT".to_string(),
            A2AMessageKinds::GetMessages => "GET_MSGS".to_string(),
            A2AMessageKinds::GetMessagesByConnections => "GET_MSGS_BY_CONNS".to_string(),
            A2AMessageKinds::UpdateMessageStatusByConnections => "UPDATE_MSG_STATUS_BY_CONNS".to_string(),
            A2AMessageKinds::MessageStatusUpdatedByConnections => "MSG_STATUS_UPDATED_BY_CONNS".to_string(),
            A2AMessageKinds::Messages => "MSGS".to_string(),
            A2AMessageKinds::UpdateConnectionStatus => "UPDATE_CONN_STATUS".to_string(),
            A2AMessageKinds::UpdateConfigs => "UPDATE_CONFIGS".to_string(),
            A2AMessageKinds::ConfigsUpdated => "CONFIGS_UPDATED".to_string(),
            A2AMessageKinds::UpdateComMethod => "UPDATE_COM_METHOD".to_string(),
            A2AMessageKinds::ComMethodUpdated => "COM_METHOD_UPDATED".to_string(),
            A2AMessageKinds::SendRemoteMessage => "SEND_REMOTE_MSG".to_string(),
            A2AMessageKinds::SendRemoteMessageResponse => "REMOTE_MSG_SENT".to_string(),
        }
    }
}

pub async fn prepare_message_for_agency(message: &A2AMessage, agency_did: &str) -> AgencyClientResult<Vec<u8>> {
    pack_for_agency_v2(message, agency_did).await
}

async fn pack_for_agency_v2(message: &A2AMessage, agency_did: &str) -> AgencyClientResult<Vec<u8>> {
    trace!("pack_for_agency_v2 >>>");
    let agent_vk = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY)?;
    let my_vk = agency_settings::get_config_value(agency_settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

    let message = ::serde_json::to_string(&message)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, format!("Cannot serialize A2A message: {}", err)))?;

    let receiver_keys = ::serde_json::to_string(&vec![&agent_vk])
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, format!("Cannot serialize receiver keys: {}", err)))?;

    let message = crypto::pack_message(Some(&my_vk), &receiver_keys, message.as_bytes()).await?;

    prepare_forward_message(message, agency_did).await
}

pub async fn parse_message_from_response(response: &Vec<u8>) -> AgencyClientResult<String> {
    let unpacked_msg = crypto::unpack_message(&response[..]).await?;

    let message: Value = ::serde_json::from_slice(unpacked_msg.as_slice())
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot deserialize response: {}", err)))?;

    Ok(message["message"].as_str()
        .ok_or(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, "Cannot find `message` field on response"))?.to_string())
}

async fn parse_response_from_agency(response: &Vec<u8>) -> AgencyClientResult<Vec<A2AMessage>> {
    trace!("parse_response_from_agency >>> processing payload of {} bytes", response.len());

    let message: String = if AgencyMockDecrypted::has_decrypted_mock_responses() {
        warn!("parse_response_from_agency_v2 >> retrieving decrypted mock response");
        AgencyMockDecrypted::get_next_decrypted_response()
    } else {
        parse_message_from_response(response).await?
    };

    trace!("AgencyComm Inbound V2 A2AMessage: {}", message);

    let message: A2AMessage = serde_json::from_str(&message)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))?;

    Ok(vec![message])
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Bundled<T> {
    bundled: Vec<T>,
}

impl<T> Bundled<T> {
    pub fn create(bundled: T) -> Bundled<T> {
        let mut vec = Vec::new();
        vec.push(bundled);
        Bundled {
            bundled: vec,
        }
    }

    pub fn encode(&self) -> AgencyClientResult<Vec<u8>> where T: serde::Serialize {
        rmp_serde::to_vec_named(self)
            .map_err(|err| {
                error!("Could not convert bundle to messagepack: {}", err);
                AgencyClientError::from_msg(AgencyClientErrorKind::InvalidMessagePack, format!("Could not encode bundle: {}", err))
            })
    }
}

pub fn try_i8_bundle(data: Vec<u8>) -> AgencyClientResult<Bundled<Vec<u8>>> {
    let bundle: Bundled<Vec<i8>> =
        rmp_serde::from_slice(&data[..])
            .map_err(|_| {
                trace!("could not deserialize bundle with i8, will try u8");
                AgencyClientError::from_msg(AgencyClientErrorKind::InvalidMessagePack, "Could not deserialize bundle with i8")
            })?;

    let mut new_bundle: Bundled<Vec<u8>> = Bundled { bundled: Vec::new() };
    for i in bundle.bundled {
        let mut buf: Vec<u8> = Vec::new();
        for j in i { buf.push(j as u8); }
        new_bundle.bundled.push(buf);
    }
    Ok(new_bundle)
}

pub fn to_u8(bytes: &Vec<i8>) -> Vec<u8> {
    bytes.iter().map(|i| *i as u8).collect()
}

pub fn to_i8(bytes: &Vec<u8>) -> Vec<i8> {
    bytes.iter().map(|i| *i as i8).collect()
}

pub fn bundle_from_u8(data: Vec<u8>) -> AgencyClientResult<Bundled<Vec<u8>>> {
    try_i8_bundle(data.clone())
        .or_else(|_| rmp_serde::from_slice::<Bundled<Vec<u8>>>(&data[..]))
        .map_err(|err| {
            error!("could not deserialize bundle with i8 or u8: {}", err);
            AgencyClientError::from_msg(AgencyClientErrorKind::InvalidMessagePack, "Could not deserialize bundle with i8 or u8")
        })
}

async fn prepare_forward_message(message: Vec<u8>, did: &str) -> AgencyClientResult<Vec<u8>> {
    trace!("prepare_forward_message >>>");
    let agency_vk = agency_settings::get_config_value(agency_settings::CONFIG_AGENCY_VERKEY)?;

    let message = ForwardV2::new(did.to_string(), message)?;

    match message {
        A2AMessage::Version2(A2AMessageV2::Forward(msg)) => prepare_forward_message_for_agency_v2(&msg, &agency_vk).await,
        _ => Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidState, "Invalid message type"))
    }
}

async fn prepare_forward_message_for_agency_v2(message: &ForwardV2, agency_vk: &str) -> AgencyClientResult<Vec<u8>> {
    let message = serde_json::to_string(message)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, format!("Cannot serialize Forward message: {}", err)))?;

    let receiver_keys = serde_json::to_string(&vec![agency_vk])
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, format!("Cannot serialize receiver keys: {}", err)))?;

    crypto::pack_message(None, &receiver_keys, message.as_bytes()).await
}

async fn prepare_message_for_agent(messages: Vec<A2AMessage>, pw_vk: &str, agent_did: &str, agent_vk: &str) -> AgencyClientResult<Vec<u8>> {
    debug!("prepare_message_for_agent >> {:?}", messages);
    let message = messages.get(0)
        .ok_or(AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, "Cannot get message"))?;

    let message = serde_json::to_string(message)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, format!("Cannot serialize A2A message: {}", err)))?;

    let receiver_keys = serde_json::to_string(&vec![&agent_vk])
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::SerializationError, format!("Cannot receiver keys: {}", err)))?;

    let message = crypto::pack_message(Some(pw_vk), &receiver_keys, message.as_bytes()).await?;

    /* forward to did */
    let message = ForwardV2::new(agent_did.to_owned(), message)?;

    let to_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

    pack_for_agency_v2(&message, &to_did).await
}

#[async_trait]
pub trait GeneralMessage {
    type Msg;

    //todo: deserialize_message

    fn to(&mut self, to_did: &str) -> AgencyClientResult<&mut Self> {
        validation::validate_did(to_did)?;
        self.set_to_did(to_did.to_string());
        Ok(self)
    }

    fn to_vk(&mut self, to_vk: &str) -> AgencyClientResult<&mut Self> {
        validation::validate_verkey(to_vk)?;
        self.set_to_vk(to_vk.to_string());
        Ok(self)
    }

    fn agent_did(&mut self, did: &str) -> AgencyClientResult<&mut Self> {
        validation::validate_did(did)?;
        self.set_agent_did(did.to_string());
        Ok(self)
    }

    fn agent_vk(&mut self, to_vk: &str) -> AgencyClientResult<&mut Self> {
        validation::validate_verkey(to_vk)?;
        self.set_agent_vk(to_vk.to_string());
        Ok(self)
    }

    fn set_to_vk(&mut self, to_vk: String);
    fn set_to_did(&mut self, to_did: String);
    fn set_agent_did(&mut self, did: String);
    fn set_agent_vk(&mut self, vk: String);

    async fn prepare_request(&mut self) -> AgencyClientResult<Vec<u8>>;
}

pub fn create_keys() -> CreateKeyBuilder { CreateKeyBuilder::create() }

pub fn delete_connection() -> DeleteConnectionBuilder { DeleteConnectionBuilder::create() }

pub fn update_data() -> UpdateProfileDataBuilder { UpdateProfileDataBuilder::create() }

pub fn get_messages() -> GetMessagesBuilder { GetMessagesBuilder::create() }

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_to_u8() {
        let vec: Vec<i8> = vec![-127, -89, 98, 117, 110, 100, 108, 101, 100, -111, -36, 5, -74];

        let buf = to_u8(&vec);
        info!("new bundle: {:?}", buf);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_to_i8() {
        let vec: Vec<u8> = vec![129, 167, 98, 117, 110, 100, 108, 101, 100, 145, 220, 19, 13];
        let buf = to_i8(&vec);
        info!("new bundle: {:?}", buf);
    }
}
