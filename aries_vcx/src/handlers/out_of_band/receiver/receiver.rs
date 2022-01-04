use std::clone::Clone;
use std::future::Future;

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
use crate::messages::connection::service::ServiceResolvable;
use std::convert::TryFrom;

#[derive(Default, Debug, PartialEq)]
pub struct OutOfBandReceiver {
    pub oob: OutOfBand
}

impl OutOfBandReceiver {
    pub fn create_from_a2a_msg(msg: &A2AMessage) -> VcxResult<Self> {
        trace!("OutOfBand::create_from_a2a_msg >>> msg: {:?}", msg);
        match msg {
            A2AMessage::OutOfBand(oob) => Ok(OutOfBandReceiver { oob: oob.clone() }),
            _ => Err(VcxError::from(VcxErrorKind::InvalidMessageFormat))
        }
    }

    pub fn connection_exists<'a>(&self, connections: &'a Vec<&'a Connection>) -> VcxResult<Option<&'a Connection>> {
        trace!("OutOfBand::connection_exists >>>");
        for service in &self.oob.services {
            for connection in connections {
                match connection.bootstrap_did_doc() {
                    Some(did_doc) => {
                        if let ServiceResolvable::Did(did) = service {
                            if did.to_string() == did_doc.id {
                                return Ok(Some(connection))
                            }
                        };
                        if did_doc.resolve_service()? == service.resolve()? {
                            return Ok(Some(connection))
                        };
                    }
                    None => break
                }
            }
        };
        Ok(None)
    }

    // TODO: There may be multiple A2AMessages in a single OoB msg
    pub fn extract_a2a_message(&self) -> VcxResult<Option<A2AMessage>> {
        trace!("OutOfBand::extract_a2a_message >>>");
        if let Some(attach) = self.oob.requests_attach.get() {
            let attach_json = self.oob.requests_attach.content()?;
            match attach.id() {
                Some(id) => match id {
                    AttachmentId::CredentialOffer => {
                        let offer: CredentialOffer = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::CredentialOffer(offer.set_parent_thread_id(&self.oob.id.0))));
                    }
                    AttachmentId::CredentialRequest => {
                        let request: CredentialRequest = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::CredentialRequest(request.set_parent_thread_id(&self.oob.id.0))));
                    }
                    AttachmentId::Credential => {
                        let credential: Credential = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::Credential(credential.set_parent_thread_id(&self.oob.id.0))));
                    }
                    AttachmentId::PresentationRequest => {
                        let request: PresentationRequest = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::PresentationRequest(request)));
                    }
                    AttachmentId::Presentation => {
                        let presentation: Presentation = serde_json::from_str(&attach_json)
                            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize attachment: {}", attach_json)))?;
                        return Ok(Some(A2AMessage::Presentation(presentation.set_parent_thread_id(&self.oob.id.0))));
                    }
                }
                None => { return Ok(None); }
            };
        };
        return Ok(None);
    }

    pub async fn build_connection(&self, autohop_enabled: bool) -> VcxResult<Connection> {
        trace!("OutOfBand::build_connection >>> autohop_enabled: {}", autohop_enabled);
        let service = match self.oob.services.get(0) {
            Some(service) => service,
            None => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidInviteDetail, "No service found in OoB message"));
            }
        };
        let invite: PairwiseInvitation = PairwiseInvitation::try_from(service)?;
        Connection::create_with_invite(&self.oob.id.0, Invitation::Pairwise(invite), autohop_enabled).await
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        self.oob.to_a2a_message()
    }


    pub fn to_string(&self) -> VcxResult<String> {
        self.oob.to_string()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: OutOfBand::from_string(oob_data)?
        })
    }
}
