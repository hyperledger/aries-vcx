use std::sync::RwLock;

use actix_web::{get, post, web, Responder};
use aries_vcx_agent::aries_vcx::{
    handlers::util::AnyInvitation,
    messages::msg_fields::protocols::{connection::invitation::Invitation, notification::ack::Ack},
    protocols::connection::{State as ConnectionState, ThinState},
};

use crate::{
    controllers::Request,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    soft_assert_eq, HarnessAgent, State,
};

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct Comment {
    comment: String,
}

fn to_backchannel_state(state: ThinState) -> State {
    match state {
        ThinState::Invitee(state) => match state {
            ConnectionState::Initial => State::Initial,
            ConnectionState::Invited => State::Invited,
            ConnectionState::Requested => State::Requested,
            ConnectionState::Responded => State::Responded,
            ConnectionState::Completed => State::Complete,
        },
        ThinState::Inviter(state) => match state {
            ConnectionState::Initial => State::Initial,
            ConnectionState::Invited => State::Invited,
            ConnectionState::Requested => State::Requested,
            ConnectionState::Responded => State::Responded,
            ConnectionState::Completed => State::Complete,
        },
    }
}

impl HarnessAgent {
    pub async fn create_connection_invitation(&self) -> HarnessResult<String> {
        let invitation = self
            .aries_agent
            .connections()
            .create_invitation(None)
            .await?;
        let id = invitation.id();
        Ok(json!({ "connection_id": id, "invitation": invitation }).to_string())
    }

    pub async fn receive_connection_invitation(&self, invite: Invitation) -> HarnessResult<String> {
        let id = self
            .aries_agent
            .connections()
            .receive_invitation(AnyInvitation::Con(invite))
            .await?;
        Ok(json!({ "connection_id": id }).to_string())
    }

    pub async fn send_connection_request(&self, id: &str) -> HarnessResult<String> {
        self.aries_agent.connections().send_request(id).await?;
        Ok(json!({ "connection_id": id }).to_string())
    }

    pub async fn accept_connection_request(&self, id: &str) -> HarnessResult<String> {
        // TODO: Handle case of multiple requests received
        if !matches!(
            self.aries_agent.connections().get_state(id)?,
            ThinState::Inviter(ConnectionState::Requested)
        ) {
            return Err(HarnessError::from_kind(
                HarnessErrorType::RequestNotReceived,
            ));
        }
        self.aries_agent.connections().send_response(id).await?;
        Ok(json!({ "connection_id": id }).to_string())
    }

    pub async fn send_connection_ack(&self, id: &str) -> HarnessResult<String> {
        self.aries_agent.connections().send_ack(id).await?;
        Ok(json!({ "connection_id": id }).to_string())
    }

    pub async fn process_connection_ack(&self, ack: Ack) -> HarnessResult<String> {
        let thid = ack.decorators.thread.thid.to_string();
        self.aries_agent.connections().process_ack(ack).await?;
        Ok(json!({ "connection_id": thid }).to_string())
    }

    pub async fn get_connection_state(&self, id: &str) -> HarnessResult<String> {
        let state = to_backchannel_state(self.aries_agent.connections().get_state(id)?);
        Ok(json!({ "state": state }).to_string())
    }

    pub async fn get_connection(&self, id: &str) -> HarnessResult<String> {
        soft_assert_eq!(self.aries_agent.connections().exists_by_id(id), true);
        Ok(json!({ "connection_id": id }).to_string())
    }
}

#[post("/create-invitation")]
pub async fn create_invitation(agent: web::Data<RwLock<HarnessAgent>>) -> impl Responder {
    agent.read().unwrap().create_connection_invitation().await
}

#[post("/receive-invitation")]
pub async fn receive_invitation(
    req: web::Json<Request<Option<Invitation>>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .receive_connection_invitation(
            req.data
                .as_ref()
                .ok_or(HarnessError::from_msg(
                    HarnessErrorType::InvalidJson,
                    "Failed to deserialize pairwise invitation",
                ))?
                .clone(),
        )
        .await
}

#[post("/accept-invitation")]
pub async fn send_request(
    req: web::Json<Request<()>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().send_connection_request(&req.id).await
}

#[post("/accept-request")]
pub async fn accept_request(
    req: web::Json<Request<()>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .accept_connection_request(&req.id)
        .await
}

#[get("/{connection_id}")]
pub async fn get_connection_state(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    let connection_state = agent
        .read()
        .unwrap()
        .get_connection_state(&path.into_inner())
        .await;
    info!("Connection state: {:?}", connection_state);
    connection_state
}

#[get("/{thread_id}")]
pub async fn get_connection(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .get_connection(&path.into_inner())
        .await
}

#[post("/send-ping")]
pub async fn send_ack(
    req: web::Json<Request<Comment>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent.read().unwrap().send_connection_ack(&req.id).await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/connection")
            .service(create_invitation)
            .service(receive_invitation)
            .service(send_request)
            .service(accept_request)
            .service(send_ack)
            .service(get_connection_state),
    )
    .service(web::scope("/response/connection").service(get_connection));
}
