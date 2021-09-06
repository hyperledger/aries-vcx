use crate::handlers::out_of_band::OutOfBand;
use crate::handlers::connection::connection::Connection;
use crate::error::prelude::*;
use crate::messages::a2a::A2AMessage;
use crate::messages::attachment::AttachmentId;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_request::CredentialRequest;
use crate::messages::issuance::credential::Credential;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::connection::invite::{Invitation, PairwiseInvitation};
use std::convert::TryFrom;

impl OutOfBand {
    pub fn create_from_a2a_msg(msg: &A2AMessage) -> VcxResult<Self> {
        match msg {
            A2AMessage::OutOfBand(oob) => Ok(oob.clone()),
            _ => Err(VcxError::from(VcxErrorKind::InvalidMessageFormat))
        }
    }

    pub fn connection_exists(&self, connections: Vec<Connection>) -> VcxResult<bool> {
        for service in &self.services {
            let full_service = service.resolve()?;
            for connection in &connections {
                let did_doc = connection.their_did_doc();
                match did_doc {
                    Some(did_doc) => {
                        for service in &did_doc.service {
                            if *service == full_service {
                                return Ok(true)
                            }
                        }
                    }
                    None => break
                }
            }
        };
        Ok(false)
    }

    // TODO: There may be multiple A2AMessages in a single OoB msg
    pub fn extract_a2a_message(&self) -> VcxResult<Option<A2AMessage>> {
        if let Some(attach) = self.requests_attach.get() {
            let attach_json = self.requests_attach.content()?;
            match attach.id() {
                Some(id) => match id {
                    AttachmentId::CredentialOffer => {
                        let offer: CredentialOffer = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::CredentialOffer(offer)));
                    }
                    AttachmentId::CredentialRequest => {
                        let request: CredentialRequest = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::CredentialRequest(request)));
                    }
                    AttachmentId::Credential => {
                        let credential: Credential = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::Credential(credential)));
                    }
                    AttachmentId::PresentationRequest => {
                        let request: PresentationRequest = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::PresentationRequest(request)));
                    }
                    AttachmentId::Presentation => {
                        let presentation: Presentation = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::Presentation(presentation)));
                    }
                }
                None => { return Ok(None); }
            };
        };
        return Ok(None);
    }

    pub fn build_connection(&self, autohop_enabled: bool) -> VcxResult<Connection> {
        let service = match self.services.get(0) {
            Some(service) => service,
            None => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidInviteDetail, "No service found in OoB message"));
            }
        };
        let invite: PairwiseInvitation = PairwiseInvitation::try_from(service)?;
        Connection::create_with_invite(&self.id.0, Invitation::Pairwise(invite), autohop_enabled)
    }
}

// TODO
// impl TryFrom<OutOfBand> for A2AMessage {
//     type Error = VcxError;
// 
//     fn try_from(OutOfBand: i32) -> Result<Self, Self::Error> {
//     }
// }
