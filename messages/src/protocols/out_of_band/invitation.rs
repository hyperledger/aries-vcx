use crate::a2a::message_type::MessageType;
use crate::a2a::{A2AMessage, MessageId};
use crate::a2a_message;
use crate::concepts::attachment::Attachments;
use crate::concepts::mime_type::MimeType;
use crate::concepts::timing::Timing;
use crate::errors::error::prelude::*;
use crate::protocols::out_of_band::service_oob::ServiceOob;
use crate::protocols::out_of_band::GoalCode;

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
    pub handshake_protocols: Option<Vec<MessageType>>, // TODO: Make a separate type
    pub services: Vec<ServiceOob>,
    #[serde(default, skip_serializing_if = "Attachments::is_empty", rename = "requests~attach")]
    pub requests_attach: Attachments,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

a2a_message!(OutOfBandInvitation);

impl OutOfBandInvitation {
    pub fn to_string(&self) -> String {
        json!(self).to_string()
    }

    pub fn from_string(oob_data: &str) -> MessagesResult<OutOfBandInvitation> {
        serde_json::from_str(oob_data).map_err(|err| {
            MessagesError::from_msg(
                MessagesErrorKind::InvalidJson,
                format!("Cannot deserialize out of band message: {:?}", err),
            )
        })
    }
}

#[cfg(test)]
pub mod unit_tests {
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_serde_no_attach() {
        let msg = r#"
            {
                "@id": "cd0f8ab1-6bb3-4de6-8d14-15e2dd2f463f",
                "@type": "https://didcomm.org/out-of-band/1.1/invitation",
                "handshake_protocols": [
                    "https://didcomm.org/connections/1.0"
                ],
                "services": [
                    "did:sov:V4SGRU86Z58d6TV7PBUe6f"
                ]
            }
        "#;

        let invite: OutOfBandInvitation = serde_json::from_str(msg).unwrap();

        assert!(!invite.to_string().contains("attach"));
        assert_eq!(invite.id.0, "cd0f8ab1-6bb3-4de6-8d14-15e2dd2f463f");
        assert_eq!(invite.handshake_protocols.unwrap().len(), 1);
        assert_eq!(invite.services.len(), 1);
    }
}
