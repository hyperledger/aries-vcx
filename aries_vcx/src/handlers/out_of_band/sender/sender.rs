use crate::handlers::out_of_band::{OutOfBand, GoalCode};
use crate::messages::attachment::{AttachmentId, AttachmentEncoding};
use crate::messages::connection::service::ServiceResolvable;
use crate::messages::a2a::A2AMessage;
use crate::error::prelude::*;

impl OutOfBand {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn set_goal_code(mut self, goal_code: GoalCode) -> Self {
        self.goal_code = Some(goal_code);
        self
    }

    pub fn set_goal(mut self, goal: &str) -> Self {
        self.goal = Some(goal.to_string());
        self
    }

    pub fn append_service(mut self, service: ServiceResolvable) -> Self {
        self.services.push(service);
        self
    }

    pub fn append_a2a_message(mut self, msg: A2AMessage) -> VcxResult<Self> {
        let (attach_id, attach) = match msg {
            A2AMessage::CredentialRequest(request) => {
                (AttachmentId::CredentialRequest,
                serde_json::to_string(&request)
                    .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize request: {:?}", request)))?)
            }
            A2AMessage::PresentationRequest(request) => {
                (AttachmentId::PresentationRequest,
                serde_json::to_string(&request)
                    .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize request: {:?}", request)))?)
            }
            A2AMessage::CredentialOffer(offer) => {
                (AttachmentId::CredentialOffer,
                serde_json::to_string(&offer)
                    .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize offer: {:?}", offer)))?)
            }
            A2AMessage::Credential(credential) => {
                (AttachmentId::Credential,
                serde_json::to_string(&credential)
                    .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize credential: {:?}", credential)))?)
            }
            A2AMessage::Presentation(presentation) => {
                (AttachmentId::Presentation,
                serde_json::to_string(&presentation)
                    .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize presentation: {:?}", presentation)))?)
            }
            _ => {
                return Err(VcxError::from(VcxErrorKind::InvalidMessageFormat))
            }
        };
        self.requests_attach.add_base64_encoded_json_attachment(attach_id, ::serde_json::Value::String(attach))?;
        Ok(self)
    }
}
