use crate::a2a_message;
use crate::error::prelude::*;
use crate::a2a::message_type::MessageType;
use crate::a2a::{A2AMessage, MessageId};
use crate::attachment::Attachments;
use crate::did_doc::service_resolvable::ServiceResolvable;
use crate::mime_type::MimeType;
use crate::timing::Timing;
use crate::out_of_band::GoalCode;

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
    pub services: Vec<ServiceResolvable>,
    #[serde(rename = "requests~attach")]
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
                MesssagesErrorKind::InvalidJson,
                format!("Cannot deserialize out of band message: {:?}", err),
            )
        })
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {

    use crate::did_doc::service_aries::AriesService;
    use super::*;
    use crate::did_doc::test_utils::*;


    pub fn _oob_invitation() -> OutOfBandInvitation {
          OutOfBandInvitation {
            id: Default::default(),
            label: None,
            goal_code: None,
            goal: None,
            accept: None,
            handshake_protocols: None,
            services : vec![_create_service()],

            requests_attach: Default::default(),
            timing: None,
        }

    }
    fn _create_service() -> ServiceResolvable {
        ServiceResolvable::AriesService(
            AriesService::create()
                .set_service_endpoint(_service_endpoint())
                .set_routing_keys(_routing_keys())
                .set_recipient_keys(vec![_key_4()]),
        )
    }
}

