pub mod receiver;
pub mod sender;

// TODO: move to messages
use crate::messages::mime_type::MimeType;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::a2a::message_type::MessageType;
use crate::messages::connection::service::ServiceResolvable;
use crate::messages::attachment::Attachments;
use crate::error::prelude::*;
use crate::a2a_message;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GoalCode {
    #[serde(rename = "issue-vc")]
    IssueVC,
    #[serde(rename = "request-proof")]
    RequestProof,
    #[serde(rename = "create-account")]
    CreateAccount,
    #[serde(rename = "p2p-messaging")]
    P2PMessaging
}

pub enum HandshakeProtocol {
    ConnectionV1,
    DidExchangeV1
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct OutOfBand {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<GoalCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<MimeType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<MessageType>>,
    pub services: Vec<ServiceResolvable>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Attachments,
}

a2a_message!(OutOfBand);

impl OutOfBand {
    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize out of band message: {:?}", err)))
    }

    pub fn from_string(oob_data: &str) -> VcxResult<OutOfBand> {
        serde_json::from_str(oob_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize out of band message: {:?}", err)))
    }
}

// TODO: Add more tests
#[cfg(test)]
mod test {
    use super::*;

    use crate::messages::connection::service::FullService;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_oob_serialize_deserialize() {
        let mut oob = OutOfBand::create()
            .set_label("test")
            .set_goal("test")
            .set_goal_code(&GoalCode::P2PMessaging);
        oob.append_service(&ServiceResolvable::FullService(FullService::default())).unwrap();
        let serialized_oob = oob.to_string().unwrap();
        let deserialized_oob = OutOfBand::from_string(&serialized_oob).unwrap();
        assert_eq!(oob, deserialized_oob);
    }
}
