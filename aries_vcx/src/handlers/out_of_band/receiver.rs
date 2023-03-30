use std::clone::Clone;
use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use diddoc::aries::diddoc::AriesDidDoc;
use messages2::decorators::attachment::AttachmentType;
use messages2::decorators::thread::Thread;
use messages2::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;
use messages2::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages2::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;
use messages2::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages2::msg_fields::protocols::out_of_band::invitation::{Invitation, OobService};
use messages2::msg_fields::protocols::out_of_band::OutOfBand;
use messages2::msg_fields::protocols::present_proof::present::Presentation;
use messages2::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages2::msg_fields::protocols::present_proof::PresentProof;
use messages2::AriesMessage;
use serde::Deserialize;

use crate::common::ledger::transactions::resolve_service;
use crate::core::profile::profile::Profile;
use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::util::{AnyInvitation, AttachmentId};
use crate::protocols::connection::GenericConnection;

#[derive(Debug, PartialEq, Clone)]
pub struct OutOfBandReceiver {
    pub oob: Invitation,
}

impl OutOfBandReceiver {
    pub fn create_from_a2a_msg(msg: &AriesMessage) -> VcxResult<Self> {
        trace!("OutOfBandReceiver::create_from_a2a_msg >>> msg: {:?}", msg);
        match msg {
            AriesMessage::OutOfBand(OutOfBand::Invitation(oob)) => Ok(OutOfBandReceiver { oob: oob.clone() }),
            m => Err(AriesVcxError::from_msg(AriesVcxErrorKind::InvalidMessageFormat,
                                                 format!("Expected OutOfBandInvitation message to create OutOfBandReceiver, but received message of unknown type: {:?}", m))),
        }
    }

    pub fn get_id(&self) -> String {
        self.oob.id.clone()
    }

    pub async fn connection_exists<'a>(
        &self,
        profile: &Arc<dyn Profile>,
        connections: &'a Vec<&'a MediatedConnection>,
    ) -> VcxResult<Option<&'a MediatedConnection>> {
        trace!("OutOfBandReceiver::connection_exists >>>");
        for service in &self.oob.content.services {
            for connection in connections {
                match connection.bootstrap_did_doc() {
                    Some(did_doc) => {
                        if let OobService::Did(did) = service {
                            if did.to_string() == did_doc.id {
                                return Ok(Some(connection));
                            }
                        };
                        if did_doc.get_service()? == resolve_service(profile, service).await? {
                            return Ok(Some(connection));
                        };
                    }
                    None => break,
                }
            }
        }
        Ok(None)
    }

    pub async fn nonmediated_connection_exists<'a, I, T>(&self, profile: &Arc<dyn Profile>, connections: I) -> Option<T>
    where
        I: IntoIterator<Item = (T, &'a GenericConnection)> + Clone,
    {
        trace!("OutOfBandReceiver::connection_exists >>>");

        for service in &self.oob.content.services {
            for (idx, connection) in connections.clone() {
                if Self::connection_matches_service(profile, connection, service).await {
                    return Some(idx);
                }
            }
        }

        None
    }

    async fn connection_matches_service(
        profile: &Arc<dyn Profile>,
        connection: &GenericConnection,
        service: &OobService,
    ) -> bool {
        match connection.bootstrap_did_doc() {
            None => false,
            Some(did_doc) => Self::did_doc_matches_service(profile, service, did_doc).await,
        }
    }

    async fn did_doc_matches_service(profile: &Arc<dyn Profile>, service: &OobService, did_doc: &AriesDidDoc) -> bool {
        // Ugly, but it's best to short-circuit.
        Self::did_doc_matches_service_did(service, did_doc)
            || Self::did_doc_matches_resolved_service(profile, service, did_doc)
                .await
                .unwrap_or(false)
    }

    fn did_doc_matches_service_did(service: &OobService, did_doc: &AriesDidDoc) -> bool {
        match service {
            OobService::Did(did) => did.to_string() == did_doc.id,
            _ => false,
        }
    }

    async fn did_doc_matches_resolved_service(
        profile: &Arc<dyn Profile>,
        service: &OobService,
        did_doc: &AriesDidDoc,
    ) -> VcxResult<bool> {
        let did_doc_service = did_doc.get_service()?;
        let oob_service = resolve_service(profile, service).await?;

        Ok(did_doc_service == oob_service)
    }

    // TODO: There may be multiple A2AMessages in a single OoB msg
    pub fn extract_a2a_message(&self) -> VcxResult<Option<AriesMessage>> {
        trace!("OutOfBandReceiver::extract_a2a_message >>>");
        if let Some(attach) = self.oob.content.requests_attach.get(0) {
            let AttachmentType::Json(attach_json) = &attach.data.content else {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Attachment is not JSON: {:?}", attach),
                ));
            };

            let attach_id = if let Some(attach_id) = attach.id.as_deref() {
                let attach_id: AttachmentId = serde_json::from_str(attach_id).map_err(|err| {
                    AriesVcxError::from_msg(
                        AriesVcxErrorKind::SerializationError,
                        format!("Failed to deserialize attachment ID: {}", err),
                    )
                })?;

                Some(attach_id)
            } else {
                None
            };

            match attach_id {
                Some(id) => match id {
                    AttachmentId::CredentialOffer => {
                        let mut offer = OfferCredential::deserialize(attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;

                        if let Some(thread) = &mut offer.decorators.thread {
                            thread.pthid = Some(self.oob.id.clone());
                        } else {
                            let mut thread = Thread::new(offer.id.clone());
                            thread.pthid = Some(self.oob.id.clone());
                            offer.decorators.thread = Some(thread);
                        }

                        return Ok(Some(AriesMessage::CredentialIssuance(
                            CredentialIssuance::OfferCredential(offer),
                        )));
                    }
                    AttachmentId::CredentialRequest => {
                        let mut request = RequestCredential::deserialize(attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;

                        if let Some(thread) = &mut request.decorators.thread {
                            thread.pthid = Some(self.oob.id.clone());
                        } else {
                            let mut thread = Thread::new(request.id.clone());
                            thread.pthid = Some(self.oob.id.clone());
                            request.decorators.thread = Some(thread);
                        }

                        return Ok(Some(AriesMessage::CredentialIssuance(
                            CredentialIssuance::RequestCredential(request),
                        )));
                    }
                    AttachmentId::Credential => {
                        let mut credential = IssueCredential::deserialize(attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;

                        credential.decorators.thread.pthid = Some(self.oob.id.clone());

                        return Ok(Some(AriesMessage::CredentialIssuance(
                            CredentialIssuance::IssueCredential(credential),
                        )));
                    }
                    AttachmentId::PresentationRequest => {
                        let request = RequestPresentation::deserialize(attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;

                        return Ok(Some(AriesMessage::PresentProof(PresentProof::RequestPresentation(
                            request,
                        ))));
                    }
                    AttachmentId::Presentation => {
                        let mut presentation = Presentation::deserialize(attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;

                        presentation.decorators.thread.pthid = Some(self.oob.id.clone());

                        return Ok(Some(AriesMessage::PresentProof(PresentProof::Presentation(
                            presentation,
                        ))));
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
        did_doc: AriesDidDoc,
        autohop_enabled: bool,
    ) -> VcxResult<MediatedConnection> {
        trace!(
            "OutOfBandReceiver::build_connection >>> autohop_enabled: {}",
            autohop_enabled
        );
        MediatedConnection::create_with_invite(
            &self.oob.id,
            profile,
            agency_client,
            AnyInvitation::Oob(self.oob.clone()),
            did_doc,
            autohop_enabled,
        )
        .await
    }

    pub fn to_aries_message(&self) -> AriesMessage {
        self.oob.clone().into()
    }

    pub fn to_string(&self) -> String {
        json!(self.oob).to_string()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: serde_json::from_str(oob_data)?,
        })
    }
}
