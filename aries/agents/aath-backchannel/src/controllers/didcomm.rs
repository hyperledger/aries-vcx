use std::sync::RwLock;

use actix_web::{web, HttpResponse, Responder};
use aries_vcx_agent::aries_vcx::{
    messages::{
        msg_fields::protocols::{
            connection::Connection,
            cred_issuance::{v1::CredentialIssuanceV1, CredentialIssuance},
            did_exchange::{
                v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, v1_x::request::AnyRequest,
                DidExchange,
            },
            notification::Notification,
            present_proof::{v1::PresentProofV1, PresentProof},
            trust_ping::TrustPing,
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
            m => {
                warn!("Received unexpected connection protocol message: {:?}", m);
            }
        };
        Ok(())
    }

    async fn handle_issuance_msg(
        &self,
        msg: CredentialIssuance,
        connection_id: &str,
    ) -> HarnessResult<()> {
        match msg {
            CredentialIssuance::V1(msg) => self.handle_issuance_msg_v1(msg, connection_id).await,
            CredentialIssuance::V2(_) => {
                unimplemented!("V2 issuance is not implemented for aries-vcx aath")
            }
        }
    }

    async fn handle_issuance_msg_v1(
        &self,
        msg: CredentialIssuanceV1,
        connection_id: &str,
    ) -> HarnessResult<()> {
        match msg {
            CredentialIssuanceV1::OfferCredential(offer) => {
                self.aries_agent
                    .holder()
                    .create_from_offer(connection_id, offer.clone())?;
            }
            CredentialIssuanceV1::ProposeCredential(proposal) => {
                self.aries_agent
                    .issuer()
                    .accept_proposal(connection_id, &proposal)
                    .await?;
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
            m => {
                warn!("Received unexpected issuance protocol message: {:?}", m);
            }
        };
        Ok(())
    }

    async fn handle_presentation_msg(
        &self,
        msg: PresentProof,
        connection_id: &str,
    ) -> HarnessResult<()> {
        match msg {
            PresentProof::V1(msg) => self.handle_presentation_msg_v1(msg, connection_id).await,
            PresentProof::V2(_) => {
                unimplemented!("V2 issuance is not implemented for aries-vcx aath")
            }
        }
    }

    async fn handle_presentation_msg_v1(
        &self,
        msg: PresentProofV1,
        connection_id: &str,
    ) -> HarnessResult<()> {
        match msg {
            PresentProofV1::RequestPresentation(request) => {
                self.aries_agent
                    .prover()
                    .create_from_request(connection_id, request)?;
            }
            PresentProofV1::Presentation(presentation) => {
                let thread_id = presentation.decorators.thread.thid.clone();
                self.aries_agent
                    .verifier()
                    .verify_presentation(&thread_id, presentation)
                    .await?;
            }
            m => {
                // todo: use {} display formatter
                warn!("Received unexpected presentation protocol message: {:?}", m);
            }
        };
        Ok(())
    }

    async fn handle_did_exchange_msg(
        &self,
        msg: DidExchange,
        recipient_verkey: String,
    ) -> HarnessResult<()> {
        match msg {
            DidExchange::V1_0(DidExchangeV1_0::Request(request)) => {
                self.queue_didexchange_request(AnyRequest::V1_0(request), recipient_verkey)?;
            }
            DidExchange::V1_1(DidExchangeV1_1::Request(request)) => {
                self.queue_didexchange_request(AnyRequest::V1_1(request), recipient_verkey)?;
            }
            DidExchange::V1_0(DidExchangeV1_0::Response(response)) => {
                let res = self
                    .aries_agent
                    .did_exchange()
                    .handle_msg_response(response.into())
                    .await;
                if let Err(err) = res {
                    error!("Error sending complete: {:?}", err);
                };
            }
            DidExchange::V1_1(DidExchangeV1_1::Response(response)) => {
                let res = self
                    .aries_agent
                    .did_exchange()
                    .handle_msg_response(response.into())
                    .await;
                if let Err(err) = res {
                    error!("Error sending complete: {:?}", err);
                };
            }
            DidExchange::V1_0(DidExchangeV1_0::Complete(complete))
            | DidExchange::V1_1(DidExchangeV1_1::Complete(complete)) => {
                self.aries_agent
                    .did_exchange()
                    .handle_msg_complete(complete)?;
            }
            DidExchange::V1_0(DidExchangeV1_0::ProblemReport(problem_report))
            | DidExchange::V1_1(DidExchangeV1_1::ProblemReport(problem_report)) => {
                self.aries_agent
                    .did_exchange()
                    .receive_problem_report(problem_report)?;
            }
        };
        Ok(())
    }

    pub async fn receive_message(&self, payload: Vec<u8>) -> HarnessResult<HttpResponse> {
        let (message, sender_vk, recipient_vk) = EncryptionEnvelope::unpack_aries_msg(
            self.aries_agent.wallet().as_ref(),
            &payload,
            &None,
        )
        .await?;
        let sender_vk = sender_vk.ok_or_else(|| {
            HarnessError::from_msg(
                HarnessErrorType::EncryptionError,
                "Received anoncrypted message",
            )
        })?;

        info!("Received message: {}", message);
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
            AriesMessage::TrustPing(TrustPing::Ping(msg)) => {
                let connection_id = self
                    .aries_agent
                    .connections()
                    .get_by_sender_vk(sender_vk.base58())?;
                self.aries_agent
                    .connections()
                    .process_trust_ping(msg, &connection_id)
                    .await?
            }
            AriesMessage::Connection(msg) => self.handle_connection_msg(msg).await?,
            AriesMessage::CredentialIssuance(msg) => {
                let connection_id = self
                    .aries_agent
                    .connections()
                    .get_by_sender_vk(sender_vk.base58())?;
                self.handle_issuance_msg(msg, &connection_id).await?
            }
            AriesMessage::DidExchange(msg) => {
                self.handle_did_exchange_msg(msg, recipient_vk.base58())
                    .await?
            }
            AriesMessage::PresentProof(msg) => {
                let connection_id = self
                    .aries_agent
                    .connections()
                    .get_by_sender_vk(sender_vk.base58())?;
                self.handle_presentation_msg(msg, &connection_id).await?
            }
            m => {
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
