use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use super::OobGoalCode;
use crate::{
    decorators::{Attachment, Timing},
    misc::mime_type::MimeType,
    msg_types::types::out_of_band::OutOfBandV1_1Kind,
    protocols::common::service::Service,
    Message,
};

pub type Invitation = Message<InvitationContent, InvitationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "OutOfBandV1_1Kind::Invitation")]
pub struct InvitationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<OobGoalCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<Vec<MimeType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake_protocols: Option<Vec<String>>, // TODO: Protocol Registry
    pub services: Vec<Service>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Vec<Attachment>,
}

impl InvitationContent {
    pub fn new(services: Vec<Service>, requests_attach: Vec<Attachment>) -> Self {
        Self {
            label: None,
            goal_code: None,
            goal: None,
            accept: None,
            handshake_protocols: None,
            services,
            requests_attach,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct InvitationDecorators {
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
