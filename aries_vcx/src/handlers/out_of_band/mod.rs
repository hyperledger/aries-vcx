pub mod receiver;
pub mod sender;

// TODO: move to messages
use crate::messages::mime_type::MimeType;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::a2a::message_family::MessageFamilies;
use crate::messages::a2a::message_type::MessageType;
use crate::messages::connection::service::ServiceResolvable;
use crate::messages::attachment::{AttachmentId, Attachments};
use crate::handlers::connection::public_agent::PublicAgent;
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
    #[serde(rename = "requests~attach", skip_serializing_if = "Attachments::is_empty")]
    pub requests_attach: Attachments,
}

a2a_message!(OutOfBand);

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::utils::devsetup::*;
    use crate::messages::connection::service::FullService;
    use std::convert::TryFrom;

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_connection_reusable() {
        let _setup = SetupLibraryAgencyV2::init();
        let agent = PublicAgent::create("public-agent", "2hoqvcwupRTUNkXn6ArYzs").unwrap();
        let service = FullService::try_from(agent).unwrap();
        let oob_msg = OutOfBand::create()
            .set_label("test-label")
            .set_goal_code(GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(ServiceResolvable::FullService(service));
    }
}
