#![crate_name = "agency_client"]
extern crate async_std;
extern crate async_trait;
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

use std::u8;

use async_trait::async_trait;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use messages::a2a_message::Client2AgencyMessage;
use messages::forward::ForwardV2;
use messages::get_messages::GetMessagesBuilder;
use messages::update_connection::DeleteConnectionBuilder;

use crate::messages::create_key::CreateKeyBuilder;
use crate::testing::mocking::AgencyMockDecrypted;

use self::error::prelude::*;
use self::utils::libindy::crypto;
use self::utils::validation;

pub mod get_message;
pub mod utils;
pub mod update_connection;
pub mod update_message;
pub mod message_type;
#[macro_use]
pub mod agency_settings;
pub mod agency_client;
pub mod agent_utils;
pub mod error;
pub mod messages;
pub mod testing;
pub mod httpclient;
pub mod create_keys;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageStatusCode {
    Received,
    Reviewed,
}

impl std::string::ToString for MessageStatusCode {
    fn to_string(&self) -> String {
        match self {
            MessageStatusCode::Received => "MS-103",
            MessageStatusCode::Reviewed => "MS-106",
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
            Some("MS-103") => Ok(MessageStatusCode::Received),
            Some("MS-106") => Ok(MessageStatusCode::Reviewed),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

pub async fn prepare_message_for_agency(message: &Client2AgencyMessage, agency_did: &str) -> AgencyClientResult<Vec<u8>> {
    pack_for_agency_v2(message, agency_did).await
}

async fn pack_for_agency_v2(message: &Client2AgencyMessage, agency_did: &str) -> AgencyClientResult<Vec<u8>> {
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

async fn parse_response_from_agency(response: &Vec<u8>) -> AgencyClientResult<Vec<Client2AgencyMessage>> {
    trace!("parse_response_from_agency >>> processing payload of {} bytes", response.len());

    let message: String = if AgencyMockDecrypted::has_decrypted_mock_responses() {
        warn!("parse_response_from_agency_v2 >> retrieving decrypted mock response");
        AgencyMockDecrypted::get_next_decrypted_response()
    } else {
        parse_message_from_response(response).await?
    };

    trace!("AgencyComm Inbound V2 A2AMessage: {}", message);

    let message: Client2AgencyMessage = serde_json::from_str(&message)
        .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))?;

    Ok(vec![message])
}

async fn prepare_forward_message(message: Vec<u8>, did: &str) -> AgencyClientResult<Vec<u8>> {
    trace!("prepare_forward_message >>>");
    let agency_vk = agency_settings::get_config_value(agency_settings::CONFIG_AGENCY_VERKEY)?;

    let message = ForwardV2::new(did.to_string(), message)?;

    match message {
        Client2AgencyMessage::Forward(msg) => prepare_forward_message_for_agency_v2(&msg, &agency_vk).await,
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

async fn prepare_message_for_agent(messages: Vec<Client2AgencyMessage>, pw_vk: &str, agent_did: &str, agent_vk: &str) -> AgencyClientResult<Vec<u8>> {
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
