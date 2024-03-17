use std::sync::RwLock;

use actix_web::{get, post, web, Responder};
use aries_vcx_agent::aries_vcx::{
    did_parser::Did,
    messages::{msg_fields::protocols::did_exchange::DidExchange, AriesMessage},
    protocols::did_exchange::state_machine::requester::helpers::invitation_get_first_did_service,
};

use crate::{
    controllers::Request,
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
        let did_inviter: Did = invitation_get_first_did_service(&invitation)?;
        let (thid, pthid) = self
            .aries_agent
            .did_exchange()
            .handle_msg_invitation(did_inviter.to_string(), Some(invitation_id))
            .await?;
        if let Some(ref pthid) = pthid {
            self.store_mapping_pthid_thid(pthid.clone(), thid.clone());
        } else {
            warn!(
                "didx_requester_send_request >> No storing pthid->this mapping; no pthid available"
            );
        }
        let connection_id = pthid.unwrap_or(thid);
        Ok(json!({ "connection_id" : connection_id }).to_string())
    }

    pub async fn store_didcomm_message(&self, msg: AriesMessage) -> HarnessResult<String> {
        let mut msg_buffer = self.didx_msg_buffer.write().or_else(|_| {
            Err(HarnessError::from_msg(
                HarnessErrorType::InvalidState,
                "Failed to lock message buffer",
            ))
        })?;
        msg_buffer.push(msg);
        Ok(json!({ "status": "ok" }).to_string())
    }

    // Note: used by test-case did-exchange @T005-RFC0023
    pub async fn didx_requester_send_request_from_resolvable_did(
        &self,
        req: &CreateResolvableDidRequest,
    ) -> HarnessResult<String> {
        let (thid, pthid) = self
            .aries_agent
            .did_exchange()
            .handle_msg_invitation(req.their_public_did.clone(), None) // todo: separate the case with/without invitation on did_exchange handler
            .await?;
        let connection_id = pthid.unwrap_or(thid);
        Ok(json!({ "connection_id": connection_id }).to_string())
    }

    // While in real-life setting, requester would send the request to a service resolved from DID
    // Document AATH play role of "mediator" such that it explicitly takes request from
    // requester and passes it to responder (eg. this method)
    pub async fn didx_responder_receive_request_from_resolvable_did(
        &self,
    ) -> HarnessResult<String> {
        let request = {
            debug!("receive_did_exchange_request_resolvable_did >>");
            let msgs = self.didx_msg_buffer.write().or_else(|_| {
                Err(HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Failed to lock message buffer",
                ))
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
        if let AriesMessage::DidExchange(DidExchange::Request(ref request)) = request {
            let thid = request.decorators.thread.clone().unwrap().thid;
            Ok(json!({ "connection_id": thid }).to_string())
        } else {
            Err(HarnessError::from_msg(
                HarnessErrorType::InvalidState,
                "Message is not a request",
            ))
        }
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
            let mut request_guard = self.didx_msg_buffer.write().or_else(|_| {
                Err(HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "Failed to lock message buffer",
                ))
            })?;
            request_guard.pop().ok_or_else(|| {
                HarnessError::from_msg(
                    HarnessErrorType::InvalidState,
                    "send_did_exchange_response >> Expected to find DidExchange request message \
                     in buffer, found nothing.",
                )
            })?
        };
        if let AriesMessage::DidExchange(DidExchange::Request(request)) = request {
            let opt_invitation = match request.decorators.thread.clone().unwrap().pthid {
                None => None,
                Some(pthid) => {
                    let invitation = self.aries_agent.out_of_band().get_invitation(&pthid)?;
                    Some(invitation)
                }
            };
            let (thid, pthid) = self
                .aries_agent
                .did_exchange()
                .handle_msg_request(request.clone().into(), opt_invitation)
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
            Ok(json!({ "connection_id": thid }).to_string())
        } else {
            Err(HarnessError::from_msg(
                HarnessErrorType::InvalidState,
                "Message is not a request",
            ))
        }
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
    req: web::Json<Request<()>>,
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
    req: web::Json<Request<Option<CreateResolvableDidRequest>>>,
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
    _req: web::Json<Request<()>>,
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
