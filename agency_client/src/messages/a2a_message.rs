use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

use crate::messages::{
    connect::{Connect, ConnectResponse},
    create_agent::{CreateAgent, CreateAgentResponse},
    create_key::{CreateKey, CreateKeyResponse},
    forward::ForwardV2,
    get_messages::{GetMessages, GetMessagesResponse},
    message_type::{MessageFamilies, MessageType},
    sign_up::{SignUp, SignUpResponse},
    update_com_method::{ComMethodUpdated, UpdateComMethod},
    update_connection::{UpdateConnection, UpdateConnectionResponse},
    update_message::{UpdateMessageStatusByConnections, UpdateMessageStatusByConnectionsResponse},
};

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Client2AgencyMessage {
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

impl<'de> Deserialize<'de> for Client2AgencyMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
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
            "FWD" => ForwardV2::deserialize(value)
                .map(Client2AgencyMessage::Forward)
                .map_err(de::Error::custom),
            "CONNECT" => Connect::deserialize(value)
                .map(Client2AgencyMessage::Connect)
                .map_err(de::Error::custom),
            "CONNECTED" => ConnectResponse::deserialize(value)
                .map(Client2AgencyMessage::ConnectResponse)
                .map_err(de::Error::custom),
            "SIGNUP" => SignUp::deserialize(value)
                .map(Client2AgencyMessage::SignUp)
                .map_err(de::Error::custom),
            "SIGNED_UP" => SignUpResponse::deserialize(value)
                .map(Client2AgencyMessage::SignUpResponse)
                .map_err(de::Error::custom),
            "CREATE_AGENT" => CreateAgent::deserialize(value)
                .map(Client2AgencyMessage::CreateAgent)
                .map_err(de::Error::custom),
            "AGENT_CREATED" => CreateAgentResponse::deserialize(value)
                .map(Client2AgencyMessage::CreateAgentResponse)
                .map_err(de::Error::custom),
            "CREATE_KEY" => CreateKey::deserialize(value)
                .map(Client2AgencyMessage::CreateKey)
                .map_err(de::Error::custom),
            "KEY_CREATED" => CreateKeyResponse::deserialize(value)
                .map(Client2AgencyMessage::CreateKeyResponse)
                .map_err(de::Error::custom),
            "GET_MSGS" => GetMessages::deserialize(value)
                .map(Client2AgencyMessage::GetMessages)
                .map_err(de::Error::custom),
            "MSGS" => GetMessagesResponse::deserialize(value)
                .map(Client2AgencyMessage::GetMessagesResponse)
                .map_err(de::Error::custom),
            "UPDATE_CONN_STATUS" => UpdateConnection::deserialize(value)
                .map(Client2AgencyMessage::UpdateConnection)
                .map_err(de::Error::custom),
            "CONN_STATUS_UPDATED" => UpdateConnectionResponse::deserialize(value)
                .map(Client2AgencyMessage::UpdateConnectionResponse)
                .map_err(de::Error::custom),
            "UPDATE_MSG_STATUS_BY_CONNS" => UpdateMessageStatusByConnections::deserialize(value)
                .map(Client2AgencyMessage::UpdateMessageStatusByConnections)
                .map_err(de::Error::custom),
            "MSG_STATUS_UPDATED_BY_CONNS" => UpdateMessageStatusByConnectionsResponse::deserialize(value)
                .map(Client2AgencyMessage::UpdateMessageStatusByConnectionsResponse)
                .map_err(de::Error::custom),
            "UPDATE_COM_METHOD" => UpdateComMethod::deserialize(value)
                .map(Client2AgencyMessage::UpdateComMethod)
                .map_err(de::Error::custom),
            "COM_METHOD_UPDATED" => ComMethodUpdated::deserialize(value)
                .map(Client2AgencyMessage::ComMethodUpdated)
                .map_err(de::Error::custom),
            _ => Err(de::Error::custom("Unexpected @type field structure.")),
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

#[cfg(feature = "general_test")]
#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{
        messages::{
            a2a_message::{A2AMessageKinds, Client2AgencyMessage},
            get_messages::GetMessages,
        },
        testing::test_utils::SetupMocks,
    };

    #[test]
    fn test_serialize_deserialize_agency_message() {
        let _setup = SetupMocks::init();
        let msg = Client2AgencyMessage::GetMessages(GetMessages::build(
            A2AMessageKinds::GetMessages,
            Some("foo".into()),
            Some(vec!["abcd".into()]),
            None,
            None,
        ));
        let serialized = serde_json::to_string(&msg).unwrap();
        let expected = serde_json::to_string(&json!({
            "@type":"did:sov:123456789abcdefghi1234;spec/pairwise/1.0/GET_MSGS",
            "excludePayload":"foo",
            "uids":["abcd"]
        }))
        .unwrap();
        assert_eq!(serialized, expected);

        let deserialized: Client2AgencyMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, msg);
    }
}
