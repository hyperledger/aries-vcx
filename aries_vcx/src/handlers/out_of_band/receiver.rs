use std::clone::Clone;
use std::sync::Arc;

use agency_client::agency_client::AgencyClient;

use crate::core::profile::profile::Profile;
use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::xyz::ledger::transactions::resolve_service;
use messages::a2a::A2AMessage;
use messages::attachment::AttachmentId;
use messages::connection::invite::Invitation;
use messages::did_doc::DidDoc;
use messages::issuance::credential::Credential;
use messages::issuance::credential_offer::CredentialOffer;
use messages::issuance::credential_request::CredentialRequest;
use messages::out_of_band::invitation::OutOfBandInvitation;

use messages::proof_presentation::presentation::Presentation;
use messages::proof_presentation::presentation_request::PresentationRequest;

use messages::did_doc::service_resolvable::ServiceResolvable;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct OutOfBandReceiver {
    pub oob: OutOfBandInvitation,
}

impl OutOfBandReceiver {
    pub fn create_from_a2a_msg(msg: &A2AMessage) -> VcxResult<Self> {
        trace!("OutOfBandReceiver::create_from_a2a_msg >>> msg: {:?}", msg);
        match msg {
            A2AMessage::OutOfBandInvitation(oob) => Ok(OutOfBandReceiver { oob: oob.clone() }),
            _ => Err(VcxError::from(VcxErrorKind::InvalidMessageFormat)),
        }
    }

    pub fn get_id(&self) -> String {
        self.oob.id.0.clone()
    }

    pub async fn connection_exists<'a>(
        &self,
        profile: &Arc<dyn Profile>,
        connections: &'a Vec<&'a Connection>,
    ) -> VcxResult<Option<&'a Connection>> {
        trace!("OutOfBandReceiver::connection_exists >>>");
        for service in &self.oob.services {
            for connection in connections {
                match connection.bootstrap_did_doc().await {
                    Some(did_doc) => {
                        if let ServiceResolvable::Did(did) = service {
                            if did.to_string() == did_doc.id {
                                return Ok(Some(connection));
                            }
                        };
                        if did_doc.get_service()? == resolve_service(profile, &service).await? {
                            return Ok(Some(connection));
                        };
                    }
                    None => break,
                }
            }
        }
        Ok(None)
    }

    // TODO: There may be multiple A2AMessages in a single OoB msg
    pub fn extract_a2a_message(&self) -> VcxResult<Option<A2AMessage>> {
        trace!("OutOfBandReceiver::extract_a2a_message >>>");
        if let Some(attach) = self.oob.requests_attach.get() {
            let attach_json = self.oob.requests_attach.content()?;
            match attach.id() {
                Some(id) => match id {
                    AttachmentId::CredentialOffer => {
                        let offer: CredentialOffer = serde_json::from_str(&attach_json).map_err(|_| {
                            VcxError::from_msg(
                                VcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::CredentialOffer(
                            offer.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                    AttachmentId::CredentialRequest => {
                        let request: CredentialRequest = serde_json::from_str(&attach_json).map_err(|_| {
                            VcxError::from_msg(
                                VcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::CredentialRequest(
                            request.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                    AttachmentId::Credential => {
                        let credential: Credential = serde_json::from_str(&attach_json).map_err(|_| {
                            VcxError::from_msg(
                                VcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::Credential(
                            credential.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                    AttachmentId::PresentationRequest => {
                        let request: PresentationRequest = serde_json::from_str(&attach_json).map_err(|_| {
                            VcxError::from_msg(
                                VcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::PresentationRequest(request)));
                    }
                    AttachmentId::Presentation => {
                        let presentation: Presentation = serde_json::from_str(&attach_json).map_err(|_| {
                            VcxError::from_msg(
                                VcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::Presentation(
                            presentation.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                },
                None => {
                    return Ok(None);
                }
            };
        };
        Ok(None)
    }

    pub async fn build_connection(
        &self,
        profile: &Arc<dyn Profile>,
        agency_client: &AgencyClient,
        did_doc: DidDoc,
        autohop_enabled: bool,
    ) -> VcxResult<Connection> {
        trace!(
            "OutOfBandReceiver::build_connection >>> autohop_enabled: {}",
            autohop_enabled
        );
        Connection::create_with_invite(
            &self.oob.id.0,
            profile,
            agency_client,
            Invitation::OutOfBand(self.oob.clone()),
            did_doc,
            autohop_enabled,
        )
        .await
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        self.oob.to_a2a_message()
    }

    pub fn to_string(&self) -> String {
        self.oob.to_string()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: OutOfBandInvitation::from_string(oob_data)?,
        })
    }
}
