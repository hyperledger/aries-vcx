use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde_json::Value;

use crate::message_type::{MessageFamilies, MessageType, MessageTypes};
use crate::messages::connect::{Connect, ConnectResponse};
use crate::messages::create_agent::{CreateAgent, CreateAgentResponse};
use crate::messages::create_key::{CreateKey, CreateKeyResponse};
use crate::messages::forward::ForwardV2;
use crate::messages::get_messages::{GetMessages, GetMessagesResponse};
use crate::messages::sign_up::{SignUp, SignUpResponse};
use crate::messages::update_com_method::{ComMethodUpdated, UpdateComMethod};
use crate::messages::update_connection::{UpdateConnection, UpdateConnectionResponse};
use crate::messages::update_message::{UpdateMessageStatusByConnections, UpdateMessageStatusByConnectionsResponse};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AgencyMessageTypes {
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

    GetMessages(GetMessages),
    GetMessagesResponse(GetMessagesResponse),

    UpdateConnection(UpdateConnection),
    UpdateConnectionResponse(UpdateConnectionResponse),
    UpdateMessageStatusByConnections(UpdateMessageStatusByConnections),
    UpdateMessageStatusByConnectionsResponse(UpdateMessageStatusByConnectionsResponse),

    /// config
    UpdateComMethod(UpdateComMethod),
    ComMethodUpdated(ComMethodUpdated),
}

impl<'de> Deserialize<'de> for AgencyMessageTypes {
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
                    .map(AgencyMessageTypes::Forward)
                    .map_err(de::Error::custom)
            }
            "CONNECT" => {
                Connect::deserialize(value)
                    .map(AgencyMessageTypes::Connect)
                    .map_err(de::Error::custom)
            }
            "CONNECTED" => {
                ConnectResponse::deserialize(value)
                    .map(AgencyMessageTypes::ConnectResponse)
                    .map_err(de::Error::custom)
            }
            "SIGNUP" => {
                SignUp::deserialize(value)
                    .map(AgencyMessageTypes::SignUp)
                    .map_err(de::Error::custom)
            }
            "SIGNED_UP" => {
                SignUpResponse::deserialize(value)
                    .map(AgencyMessageTypes::SignUpResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_AGENT" => {
                CreateAgent::deserialize(value)
                    .map(AgencyMessageTypes::CreateAgent)
                    .map_err(de::Error::custom)
            }
            "AGENT_CREATED" => {
                CreateAgentResponse::deserialize(value)
                    .map(AgencyMessageTypes::CreateAgentResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_KEY" => {
                CreateKey::deserialize(value)
                    .map(AgencyMessageTypes::CreateKey)
                    .map_err(de::Error::custom)
            }
            "KEY_CREATED" => {
                CreateKeyResponse::deserialize(value)
                    .map(AgencyMessageTypes::CreateKeyResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS" => {
                GetMessages::deserialize(value)
                    .map(AgencyMessageTypes::GetMessages)
                    .map_err(de::Error::custom)
            }
            "MSGS" => {
                GetMessagesResponse::deserialize(value)
                    .map(AgencyMessageTypes::GetMessagesResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONN_STATUS" => {
                UpdateConnection::deserialize(value)
                    .map(AgencyMessageTypes::UpdateConnection)
                    .map_err(de::Error::custom)
            }
            "CONN_STATUS_UPDATED" => {
                UpdateConnectionResponse::deserialize(value)
                    .map(AgencyMessageTypes::UpdateConnectionResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_MSG_STATUS_BY_CONNS" => {
                UpdateMessageStatusByConnections::deserialize(value)
                    .map(AgencyMessageTypes::UpdateMessageStatusByConnections)
                    .map_err(de::Error::custom)
            }
            "MSG_STATUS_UPDATED_BY_CONNS" => {
                UpdateMessageStatusByConnectionsResponse::deserialize(value)
                    .map(AgencyMessageTypes::UpdateMessageStatusByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_COM_METHOD" => {
                UpdateComMethod::deserialize(value)
                    .map(AgencyMessageTypes::UpdateComMethod)
                    .map_err(de::Error::custom)
            }
            "COM_METHOD_UPDATED" => {
                ComMethodUpdated::deserialize(value)
                    .map(AgencyMessageTypes::ComMethodUpdated)
                    .map_err(de::Error::custom)
            }
            _ => Err(de::Error::custom("Unexpected @type field structure."))
        }
    }
}

// We don't want to use this anymore
#[derive(Debug)]
pub enum AgencyMsg {
    Version2(AgencyMessageTypes),
}

impl Serialize for AgencyMsg {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            AgencyMsg::Version2(msg) => msg.serialize(serializer).map_err(ser::Error::custom)
        }
    }
}

impl<'de> Deserialize<'de> for AgencyMsg {
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
                AgencyMessageTypes::deserialize(value)
                    .map(AgencyMsg::Version2)
                    .map_err(de::Error::custom)
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

    GetMessages,
    Messages,
    UpdateMessageStatusByConnections,
    MessageStatusUpdatedByConnections,
    UpdateConnectionStatus,
    UpdateComMethod,
    ComMethodUpdated,
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
            A2AMessageKinds::GetMessages => MessageFamilies::Pairwise,
            A2AMessageKinds::Messages => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateConnectionStatus => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateMessageStatusByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageStatusUpdatedByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateComMethod => MessageFamilies::Configs,
            A2AMessageKinds::ComMethodUpdated => MessageFamilies::Configs,
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
            A2AMessageKinds::GetMessages => "GET_MSGS".to_string(),
            A2AMessageKinds::UpdateMessageStatusByConnections => "UPDATE_MSG_STATUS_BY_CONNS".to_string(),
            A2AMessageKinds::MessageStatusUpdatedByConnections => "MSG_STATUS_UPDATED_BY_CONNS".to_string(),
            A2AMessageKinds::Messages => "MSGS".to_string(),
            A2AMessageKinds::UpdateConnectionStatus => "UPDATE_CONN_STATUS".to_string(),
            A2AMessageKinds::UpdateComMethod => "UPDATE_COM_METHOD".to_string(),
            A2AMessageKinds::ComMethodUpdated => "COM_METHOD_UPDATED".to_string(),
        }
    }
}
