use std::clone::Clone;
use indy_sys::PoolHandle;

use agency_client::agency_client::AgencyClient;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::handlers::out_of_band::OutOfBandInvitation;
use crate::messages::a2a::A2AMessage;
use crate::messages::attachment::AttachmentId;
use crate::messages::connection::invite::Invitation;
use crate::messages::issuance::credential::Credential;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_request::CredentialRequest;

use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;

use crate::utils::service_resolvable::ServiceResolvable;

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
        pool_handle: PoolHandle,
        connections: &'a Vec<&'a Connection>,
    ) -> VcxResult<Option<&'a Connection>> {
        trace!("OutOfBandReceiver::connection_exists >>>");
        for service in &self.oob.services {
            for connection in connections {
                match connection.bootstrap_did_doc(pool_handle) {
                    Some(did_doc) => {
                        if let ServiceResolvable::Did(did) = service {
                            if did.to_string() == did_doc.id {
                                return Ok(Some(connection));
                            }
                        };
                        if did_doc.resolve_service()? == service.resolve(pool_handle).await? {
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

    pub async fn build_connection(&self, agency_client: &AgencyClient, autohop_enabled: bool) -> VcxResult<Connection> {
        trace!(
            "OutOfBandReceiver::build_connection >>> autohop_enabled: {}",
            autohop_enabled
        );
        Connection::create_with_invite(
            &self.oob.id.0,
            agency_client.get_wallet_handle(),
            agency_client,
            Invitation::OutOfBand(self.oob.clone()),
            autohop_enabled,
        )
        .await
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        self.oob.to_a2a_message()
    }

    pub fn to_string(&self) -> VcxResult<String> {
        self.oob.to_string()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: OutOfBandInvitation::from_string(oob_data)?,
        })
    }
}
