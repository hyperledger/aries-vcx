use std::sync::RwLock;

use actix_web::{get, post, web, Responder};
use aries_vcx_agent::aries_vcx::{
    did_parser_nom::Did,
    messages::{
        msg_fields::protocols::did_exchange::{
            v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, v1_x::request::AnyRequest, DidExchange,
        },
        AriesMessage,
    },
    protocols::did_exchange::state_machine::requester::helpers::{
        invitation_get_acceptable_did_exchange_version, invitation_get_first_did_service,
    },
};
use serde_json::Value;

use crate::{
    controllers::AathRequest,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    HarnessAgent,
};

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateResolvableDidRequest {
    their_public_did: String,
    their_did: String,
}

impl HarnessAgent {
    pub async fn didx_requester_send_request(
        &self,
        invitation_id: String,
    ) -> HarnessResult<String> {
        let invitation = self
            .aries_agent
            .out_of_band()
            .get_invitation(&invitation_id)?;

        let version = invitation_get_acceptable_did_exchange_version(&invitation)?;
        let did_inviter: Did = invitation_get_first_did_service(&invitation)?;
        let (thid, pthid, my_did) = self
            .aries_agent
            .did_exchange()
            .handle_msg_invitation(did_inviter.to_string(), Some(invitation_id), version)
            .await?;
        if let Some(ref pthid) = pthid {
            self.store_mapping_pthid_thid(pthid.clone(), thid.clone());
        } else {
            warn!(
                "didx_requester_send_request >> No storing pthid->this mapping; no pthid available"
            );
        }
        let connection_id = pthid.unwrap_or(thid);
        Ok(json!({
           "connection_id" : connection_id,
           "my_did": my_did
        })
        .to_string())
    }

    pub fn queue_didexchange_request(
        &self,
        request: AnyRequest,
        recipient_verkey: String,
    ) -> HarnessResult<()> {
        info!(
            "queue_didexchange_request >> request: {:?} for recipient {}",
            request, recipient_verkey
        );

        let thid = request
            .inner()
            .decorators
            .thread
            .clone()
            .ok_or(HarnessError::from_msg(
                HarnessErrorType::InvalidState,
                "DID Exchange request is missing a thread",
            ))?;

        let mut msg_buffer = self.didx_msg_buffer.write().map_err(|_| {
            HarnessError::from_msg(
                HarnessErrorType::InvalidState,
                "Failed to lock message buffer",
            )
        })?;
        let m = AriesMessage::from(request);
        msg_buffer.push(m);

        let mut recipients = self
            .didx_thid_to_request_recipient_verkey
            .lock()
            .map_err(|_| {
                HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Failed to lock DIDExchange ",
                )
            })?;

        recipients.insert(thid.thid, recipient_verkey);

        Ok(())
    }

    // Note: used by test-case did-exchange @T005-RFC0023
    pub async fn didx_requester_send_request_from_resolvable_did(
        &self,
        req: &CreateResolvableDidRequest,
    ) -> HarnessResult<String> {
        let (thid, pthid, my_did) = self
            .aries_agent
            .did_exchange()
            .handle_msg_invitation(req.their_public_did.clone(), None, Default::default()) // todo: separate the case with/without invitation on did_exchange handler
            .await?;
        let connection_id = pthid.unwrap_or(thid);
        Ok(json!({
           "connection_id": connection_id,
           "my_did": my_did
        })
        .to_string())
    }

    // Looks up an oldest unprocessed did-exchange request message
    // Messages received via didcomm are unprocessed
    pub async fn didx_responder_receive_request_from_resolvable_did(
        &self,
    ) -> HarnessResult<String> {
        let request = {
            debug!("receive_did_exchange_request_resolvable_did >>");
            let msgs = self.didx_msg_buffer.write().map_err(|_| {
                HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Failed to lock message buffer",
                )
            })?;
            msgs.first()
                .ok_or_else(|| {
                    HarnessError::from_msg(
                        HarnessErrorType::InvalidState,
                        "receive_did_exchange_request_resolvable_did >> Expected to find \
                         DidExchange request message in buffer, found nothing.",
                    )
                })?
                .clone()
        };
        let request = match request {
            AriesMessage::DidExchange(DidExchange::V1_0(DidExchangeV1_0::Request(request)))
            | AriesMessage::DidExchange(DidExchange::V1_1(DidExchangeV1_1::Request(request))) => {
                request
            }
            _ => {
                return Err(HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Message is not a request",
                ))
            }
        };

        let thid = request.decorators.thread.clone().unwrap().thid;
        Ok(json!({ "connection_id": thid }).to_string())
    }

    // Note: AVF identifies protocols by thid, but AATH sometimes tracks identifies did-exchange
    //       connection using pthread_id instead (if one exists; eg. connection was bootstrapped
    // from invitation)       That's why we need pthid -> thid translation on AATH layer.
    fn store_mapping_pthid_thid(&self, pthid: String, thid: String) {
        info!(
            "store_mapping_pthid_thid >> pthid: {}, thid: {}",
            pthid, thid
        );
        self.didx_pthid_to_thid
            .lock()
            .unwrap()
            .insert(pthid, thid.clone());
    }

    pub async fn didx_responder_send_did_exchange_response(&self) -> HarnessResult<String> {
        let request = {
            let mut request_guard = self.didx_msg_buffer.write().map_err(|_| {
                HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Failed to lock message buffer",
                )
            })?;
            request_guard.pop().ok_or_else(|| {
                HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "send_did_exchange_response >> Expected to find DidExchange request message \
                     in buffer, found nothing.",
                )
            })?
        };
        let request = match request {
            AriesMessage::DidExchange(DidExchange::V1_0(DidExchangeV1_0::Request(r))) => {
                AnyRequest::V1_0(r)
            }
            AriesMessage::DidExchange(DidExchange::V1_1(DidExchangeV1_1::Request(r))) => {
                AnyRequest::V1_1(r)
            }
            _ => {
                return Err(HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Message is not a request",
                ))
            }
        };

        let request_thread = &request.inner().decorators.thread;

        let recipient_key = request_thread
            .as_ref()
            .and_then(|th| {
                self.didx_thid_to_request_recipient_verkey
                    .lock()
                    .unwrap()
                    .get(&th.thid)
                    .cloned()
            })
            .ok_or_else(|| {
                HarnessError::from_msg(HarnessErrorType::InvalidState, "Inviter key not found")
            })?;

        let opt_invitation = match request_thread.as_ref().and_then(|th| th.pthid.as_ref()) {
            Some(pthid) => {
                let invitation = self.aries_agent.out_of_band().get_invitation(pthid)?;
                Some(invitation)
            }
            None => None,
        };
        let (thid, pthid, my_did, their_did) = self
            .aries_agent
            .did_exchange()
            .handle_msg_request(request, recipient_key, opt_invitation)
            .await?;

        if let Some(pthid) = pthid {
            self.store_mapping_pthid_thid(pthid, thid.clone());
        } else {
            warn!("No storing pthid->this mapping; no pthid available");
        }

        self.aries_agent
            .did_exchange()
            .send_response(thid.clone())
            .await?;
        Ok(json!({
           "connection_id": thid,
           "my_did": my_did,
           "their_did": their_did
        })
        .to_string())
    }

    pub async fn didx_get_state(&self, connection_id: &str) -> HarnessResult<String> {
        let thid = match self.didx_pthid_to_thid.lock().unwrap().get(connection_id) {
            Some(thid) => {
                debug!(
                    "didx_get_state >> connection_id {} (pthid) was mapped to {} (thid)",
                    connection_id, thid
                );
                thid.clone() // connection_id was in fact pthread_id, mapping pthid -> thid exists
            }
            None => {
                connection_id.to_string() // connection_id was thid already, no mapping exists
            }
        };
        let state = self.aries_agent.did_exchange().get_state(&thid)?;
        Ok(json!({ "state": state }).to_string())
    }

    pub async fn didx_get_invitation_id(&self, connection_id: &str) -> HarnessResult<String> {
        // note: old implementation:
        // let invitation_id = self.aries_agent.did_exchange().invitation_id(id)?;

        // note: thread_id is handle for our protocol on harness level, no need to resolve anything
        Ok(json!({ "connection_id": connection_id }).to_string())
    }
}

#[post("/send-request")]
async fn send_did_exchange_request(
    req: web::Json<AathRequest<Value>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .didx_requester_send_request(req.id.clone())
        .await
}

#[post("/create-request-resolvable-did")]
async fn send_did_exchange_request_resolvable_did(
    req: web::Json<AathRequest<Option<CreateResolvableDidRequest>>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .didx_requester_send_request_from_resolvable_did(req.data.as_ref().ok_or(
            HarnessError::from_msg(
                HarnessErrorType::InvalidJson,
                "Failed to deserialize pairwise invitation",
            ),
        )?)
        .await
}

// Note: AATH expects us to identify connection-request we should have received prior to this call
//       and assign connection_id to that communication (returned from this function)
#[post("/receive-request-resolvable-did")]
async fn receive_did_exchange_request_resolvable_did(
    agent: web::Data<RwLock<HarnessAgent>>,
    _msg_buffer: web::Data<RwLock<Vec<AriesMessage>>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .didx_responder_receive_request_from_resolvable_did()
        .await
}

#[post("/send-response")]
async fn send_did_exchange_response(
    _req: web::Json<AathRequest<()>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .didx_responder_send_did_exchange_response()
        .await
}

#[get("/{thread_id}")]
async fn get_invitation_id(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .didx_get_invitation_id(&path.into_inner())
        .await
}

#[get("/{thread_id}")]
pub async fn get_did_exchange_state(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .didx_get_state(&path.into_inner())
        .await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/did-exchange")
            .service(send_did_exchange_request)
            .service(send_did_exchange_response)
            .service(receive_did_exchange_request_resolvable_did)
            .service(send_did_exchange_request_resolvable_did)
            .service(get_did_exchange_state),
    )
    .service(web::scope("/response/did-exchange").service(get_invitation_id));
}
