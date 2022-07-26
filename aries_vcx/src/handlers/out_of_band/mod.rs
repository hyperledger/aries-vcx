use crate::a2a_message;
use crate::error::prelude::*;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::a2a::message_type::MessageType;
use crate::messages::attachment::Attachments;
use crate::messages::connection::service::ServiceResolvable;
// TODO: move to messages
use crate::messages::mime_type::MimeType;

pub mod receiver;
pub mod sender;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GoalCode {
    #[serde(rename = "issue-vc")]
    IssueVC,
    #[serde(rename = "request-proof")]
    RequestProof,
    #[serde(rename = "create-account")]
    CreateAccount,
    #[serde(rename = "p2p-messaging")]
    P2PMessaging,
}

pub enum HandshakeProtocol {
    ConnectionV1,
    DidExchangeV1,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct OutOfBandInvitation {
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

a2a_message!(OutOfBandInvitation);

impl OutOfBandInvitation {
    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize out of band message: {:?}", err)))
    }

    pub fn from_string(oob_data: &str) -> VcxResult<OutOfBandInvitation> {
        serde_json::from_str(oob_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize out of band message: {:?}", err)))
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use crate::handlers::out_of_band::receiver::OutOfBandReceiver;
    use crate::handlers::out_of_band::sender::OutOfBandSender;
    use crate::messages::connection::service::FullService;
    use crate::utils::devsetup::SetupMocks;
    use crate::utils::mockdata::mockdata_oob;

    use super::*;

    #[test]
    fn test_oob_serialize_deserialize() {
        let _setup = SetupMocks::init();
        let oob_sender = OutOfBandSender::create()
            .set_label("test")
            .set_goal("test")
            .set_goal_code(&GoalCode::P2PMessaging)
            .append_service(&ServiceResolvable::FullService(FullService::default()));
        let serialized_oob = oob_sender.to_string().unwrap();
        assert_eq!(serialized_oob, mockdata_oob::ARIES_OOB_MESSAGE.replace("\n", "").replace(" ", ""));
        let deserialized_sender_oob = OutOfBandSender::from_string(&serialized_oob).unwrap();
        assert_eq!(oob_sender, deserialized_sender_oob);
        assert_eq!(oob_sender.to_a2a_message(), deserialized_sender_oob.to_a2a_message());
        let deserialized_receiver_oob = OutOfBandReceiver::from_string(&serialized_oob).unwrap();
        assert_eq!(oob_sender.to_a2a_message(), deserialized_receiver_oob.to_a2a_message());
    }
}
