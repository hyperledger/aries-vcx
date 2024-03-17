use std::sync::RwLock;

use actix_web::{web, HttpResponse, Responder};
use aries_vcx_agent::aries_vcx::{
    messages::{
        msg_fields::protocols::{
            connection::Connection,
            cred_issuance::{v1::CredentialIssuanceV1, CredentialIssuance},
            did_exchange::DidExchange,
            notification::Notification,
            present_proof::{v1::PresentProofV1, PresentProof},
        },
        AriesMessage,
    },
    utils::encryption_envelope::EncryptionEnvelope,
};

use crate::{
    error::{HarnessError, HarnessErrorType, HarnessResult},
    HarnessAgent,
};

impl HarnessAgent {
    async fn handle_connection_msg(&self, msg: Connection) -> HarnessResult<()> {
        match msg {
            Connection::Request(request) => {
                let thread_id = request
                    .decorators
                    .thread
                    .clone()
                    .map_or(request.id.clone(), |thread| thread.thid.clone());
                let connection_id = match self.aries_agent.connections().exists_by_id(&thread_id) {
                    true => thread_id,
                    false => {
                        if let Some(thread) = &request.decorators.thread {
                            if let Some(pthid) = &thread.pthid {
                                pthid.clone()
                            } else {
                                return Err(HarnessError::from_msg(
                                    HarnessErrorType::InvalidState,
                                    "Connection request does not contain parent thread id",
                                ));
                            }
                        } else {
                            return Err(HarnessError::from_msg(
                                HarnessErrorType::InvalidState,
                                "Connection request does not contain thread info decorator",
                            ));
                        }
                    }
                };
                self.aries_agent
                    .connections()
                    .accept_request(&connection_id, request)
                    .await
                    .ok();
            }
            Connection::Response(response) => {
                let thread_id = response.decorators.thread.thid.clone();
                self.aries_agent
                    .connections()
                    .accept_response(&thread_id, response)
                    .await?;
            }
            m=> {
                warn!("Received unexpected connection protocol message: {:?}", m);
            }
        };
        Ok(())
    }

    async fn handle_issuance_msg(
        &self,
        msg: CredentialIssuance,
        connection_ids: Vec<String>,
        sender_vk: &str,
    ) -> HarnessResult<()> {
        match msg {
            CredentialIssuance::V1(msg) => {
                self.handle_issuance_msg_v1(msg, connection_ids, sender_vk)
                    .await
            }
            CredentialIssuance::V2(_) => {
                unimplemented!("V2 issuance is not implemented for aries-vcx aath")
            }
        }
    }

    async fn handle_issuance_msg_v1(
        &self,
        msg: CredentialIssuanceV1,
        connection_ids: Vec<String>,
        sender_vk: &str,
    ) -> HarnessResult<()> {
        let connection_id = connection_ids.last();
        match msg {
            CredentialIssuanceV1::OfferCredential(offer) => {
                if connection_ids.len() == 1 {
                    self.aries_agent
                        .holder()
                        .create_from_offer(connection_id.unwrap(), offer.clone())?;
                } else {
                    return Err(HarnessError::from_msg(
                        HarnessErrorType::InvalidState,
                        &format!("Found multiple or no connections by verkey {}", sender_vk),
                    ));
                }
            }
            CredentialIssuanceV1::ProposeCredential(proposal) => {
                if connection_ids.len() == 1 {
                    self.aries_agent
                        .issuer()
                        .accept_proposal(connection_id.unwrap(), &proposal)
                        .await?;
                } else {
                    return Err(HarnessError::from_msg(
                        HarnessErrorType::InvalidState,
                        &format!("Found multiple or no connections by verkey {}", sender_vk),
                    ));
                }
            }
            CredentialIssuanceV1::RequestCredential(request) => {
                let thread_id = request
                    .decorators
                    .thread
                    .clone()
                    .map_or(request.id.clone(), |thread| thread.thid.clone());
                self.aries_agent
                    .issuer()
                    .process_credential_request(&thread_id, request)?;
            }
            CredentialIssuanceV1::IssueCredential(credential) => {
                let thread_id = credential.decorators.thread.thid.clone();
                self.aries_agent
                    .holder()
                    .process_credential(&thread_id, credential)
                    .await?;
            }
            m=> {
                warn!("Received unexpected issuance protocol message: {:?}", m);
            }
        };
        Ok(())
    }

    async fn handle_presentation_msg(
        &self,
        msg: PresentProof,
        connection_ids: Vec<String>,
        sender_vk: &str,
    ) -> HarnessResult<()> {
        match msg {
            PresentProof::V1(msg) => {
                self.handle_presentation_msg_v1(msg, connection_ids, sender_vk)
                    .await
            }
            PresentProof::V2(_) => {
                unimplemented!("V2 issuance is not implemented for aries-vcx aath")
            }
        }
    }

    async fn handle_presentation_msg_v1(
        &self,
        msg: PresentProofV1,
        connection_ids: Vec<String>,
        sender_vk: &str,
    ) -> HarnessResult<()> {
        let connection_id = connection_ids.last();
        match msg {
            PresentProofV1::RequestPresentation(request) => {
                if connection_ids.len() == 1 {
                    self.aries_agent
                        .prover()
                        .create_from_request(connection_id.unwrap(), request)?;
                } else {
                    return Err(HarnessError::from_msg(
                        HarnessErrorType::InvalidState,
                        &format!("Found multiple or no connections by verkey {}", sender_vk),
                    ));
                }
            }
            PresentProofV1::Presentation(presentation) => {
                let thread_id = presentation.decorators.thread.thid.clone();
                self.aries_agent
                    .verifier()
                    .verify_presentation(&thread_id, presentation)
                    .await?;
            }
            m=> {
                // todo: use {} display formatter
                warn!("Received unexpected presentation protocol message: {:?}", m);
            }
        };
        Ok(())
    }

    async fn handle_did_exchange_msg(&self, msg: DidExchange) -> HarnessResult<()> {
        match msg {
            DidExchange::Request(request) => {
                // self.aries_agent.receive_message(request.into());
                self.didx_msg_buffer.write().unwrap().push(request.into());
            }
            DidExchange::Response(response) => {
                let res = self
                    .aries_agent
                    .did_exchange()
                    .handle_msg_response(response)
                    .await;
                if let Err(err) = res {
                    error!("Error sending complete: {:?}", err);
                };
            }
            DidExchange::Complete(complete) => {
                self.aries_agent
                    .did_exchange()
                    .handle_msg_complete(complete)?;
            }
            DidExchange::ProblemReport(problem_report) => {
                self.aries_agent
                    .did_exchange()
                    .receive_problem_report(problem_report)?;
            }
        };
        Ok(())
    }

    pub async fn receive_message(&self, payload: Vec<u8>) -> HarnessResult<HttpResponse> {
        let (message, sender_vk) = EncryptionEnvelope::anon_unpack_aries_msg(
            self.aries_agent.wallet().as_ref(),
            payload.clone(),
        )
        .await?;
        let sender_vk = sender_vk.ok_or_else(|| {
            HarnessError::from_msg(
                HarnessErrorType::EncryptionError,
                "Received anoncrypted message",
            )
        })?;
        info!("Received message: {}", message);
        let connection_ids = self.aries_agent.connections().get_by_their_vk(&sender_vk)?;
        match message {
            AriesMessage::Notification(msg) => {
                match msg {
                    Notification::Ack(ack) => {
                        self.aries_agent
                            .connections()
                            .process_ack(ack.clone())
                            .await?;
                    }
                    Notification::ProblemReport(err) => {
                        error!("Received problem report: {:?}", err);
                        // todo: we should reflect this in the status of connection so aath won't
                        // keep polling
                    }
                }
            }
            AriesMessage::Connection(msg) => self.handle_connection_msg(msg).await?,
            AriesMessage::CredentialIssuance(msg) => {
                self.handle_issuance_msg(msg, connection_ids, &sender_vk)
                    .await?
            }
            AriesMessage::DidExchange(msg) => self.handle_did_exchange_msg(msg).await?,
            AriesMessage::PresentProof(msg) => {
                self.handle_presentation_msg(msg, connection_ids, &sender_vk)
                    .await?
            }
            m=> {
                warn!("Received message of unexpected type: {}", m);
            }
        };
        Ok(HttpResponse::Ok().finish())
    }
}

pub async fn receive_message(
    req: web::Bytes,
    agent: web::Data<RwLock<HarnessAgent>>,
    _msg_buffer: web::Data<RwLock<Vec<AriesMessage>>>,
) -> impl Responder {
    agent.read().unwrap().receive_message(req.to_vec()).await
}
