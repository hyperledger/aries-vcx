use core::u8;

use serde_json::Value;
use uuid::Uuid;

use crate::errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::messages::a2a_message::A2AMessageKinds;
use crate::messages::a2a_message::Client2AgencyMessage;
use crate::messages::message_type::MessageType;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ForwardV2 {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "@fwd")]
    fwd: String,
    #[serde(rename = "@msg")]
    msg: Value,
    #[serde(rename = "@id")]
    id: String,
}

impl ForwardV2 {
    pub fn new(fwd: String, msg: Vec<u8>) -> AgencyClientResult<Client2AgencyMessage> {
        let msg = serde_json::from_slice(msg.as_slice())
            .map_err(|err| AgencyClientError::from_msg(AgencyClientErrorKind::InvalidState, err))?;
        Ok(Client2AgencyMessage::Forward(ForwardV2 {
            msg_type: MessageType::build_v2(A2AMessageKinds::Forward),
            fwd,
            msg,
            id: Uuid::new_v4().to_string(),
        }))
    }
}
