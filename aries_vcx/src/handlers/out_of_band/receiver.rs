use std::{clone::Clone, sync::Arc};

use agency_client::agency_client::AgencyClient;
use messages::{
    a2a::A2AMessage,
    concepts::attachment::AttachmentId,
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::{
        connection::invite::Invitation,
        issuance::{credential::Credential, credential_offer::CredentialOffer, credential_request::CredentialRequest},
        out_of_band::{invitation::OutOfBandInvitation, service_oob::ServiceOob},
        proof_presentation::{presentation::Presentation, presentation_request::PresentationRequest},
    },
};

use crate::{
    common::ledger::transactions::resolve_service, core::profile::profile::Profile, errors::error::prelude::*,
    handlers::connection::mediated_connection::MediatedConnection, protocols::connection::GenericConnection,
};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct OutOfBandReceiver {
    pub oob: OutOfBandInvitation,
}

impl OutOfBandReceiver {
    pub fn create_from_a2a_msg(msg: &A2AMessage) -> VcxResult<Self> {
        trace!("OutOfBandReceiver::create_from_a2a_msg >>> msg: {:?}", msg);
        match msg {
            A2AMessage::OutOfBandInvitation(oob) => Ok(OutOfBandReceiver { oob: oob.clone() }),
            m => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                format!(
                    "Expected OutOfBandInvitation message to create OutOfBandReceiver, but received message of \
                     unknown type: {:?}",
                    m
                ),
            )),
        }
    }

    pub fn get_id(&self) -> String {
        self.oob.id.0.clone()
    }

    pub async fn connection_exists<'a>(
        &self,
        profile: &Arc<dyn Profile>,
        connections: &'a Vec<&'a MediatedConnection>,
    ) -> VcxResult<Option<&'a MediatedConnection>> {
        trace!("OutOfBandReceiver::connection_exists >>>");
        for service in &self.oob.services {
            for connection in connections {
                match connection.bootstrap_did_doc() {
                    Some(did_doc) => {
                        if let ServiceOob::Did(did) = service {
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

        for service in &self.oob.services {
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
        service: &ServiceOob,
    ) -> bool {
        match connection.bootstrap_did_doc() {
            None => false,
            Some(did_doc) => Self::did_doc_matches_service(profile, service, did_doc).await,
        }
    }

    async fn did_doc_matches_service(profile: &Arc<dyn Profile>, service: &ServiceOob, did_doc: &AriesDidDoc) -> bool {
        // Ugly, but it's best to short-circuit.
        Self::did_doc_matches_service_did(service, did_doc)
            || Self::did_doc_matches_resolved_service(profile, service, did_doc)
                .await
                .unwrap_or(false)
    }

    fn did_doc_matches_service_did(service: &ServiceOob, did_doc: &AriesDidDoc) -> bool {
        match service {
            ServiceOob::Did(did) => did.to_string() == did_doc.id,
            _ => false,
        }
    }

    async fn did_doc_matches_resolved_service(
        profile: &Arc<dyn Profile>,
        service: &ServiceOob,
        did_doc: &AriesDidDoc,
    ) -> VcxResult<bool> {
        let did_doc_service = did_doc.get_service()?;
        let oob_service = resolve_service(profile, service).await?;

        Ok(did_doc_service == oob_service)
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
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::CredentialOffer(
                            offer.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                    AttachmentId::CredentialRequest => {
                        let request: CredentialRequest = serde_json::from_str(&attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::CredentialRequest(
                            request.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                    AttachmentId::Credential => {
                        let credential: Credential = serde_json::from_str(&attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::Credential(
                            credential.set_parent_thread_id(&self.oob.id.0),
                        )));
                    }
                    AttachmentId::PresentationRequest => {
                        let request: PresentationRequest = serde_json::from_str(&attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
                                format!("Failed to deserialize attachment: {}", attach_json),
                            )
                        })?;
                        return Ok(Some(A2AMessage::PresentationRequest(request)));
                    }
                    AttachmentId::Presentation => {
                        let presentation: Presentation = serde_json::from_str(&attach_json).map_err(|_| {
                            AriesVcxError::from_msg(
                                AriesVcxErrorKind::SerializationError,
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
        did_doc: AriesDidDoc,
        autohop_enabled: bool,
    ) -> VcxResult<MediatedConnection> {
        trace!(
            "OutOfBandReceiver::build_connection >>> autohop_enabled: {}",
            autohop_enabled
        );
        MediatedConnection::create_with_invite(
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
