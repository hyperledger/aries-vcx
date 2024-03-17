use std::sync::RwLock;

use actix_web::{get, post, web, Responder};
use aries_vcx_agent::aries_vcx::messages::AriesMessage;

use crate::{
    controllers::Request,
    error::{HarnessError, HarnessErrorType, HarnessResult},
    soft_assert_eq, HarnessAgent,
};

impl HarnessAgent {
    pub async fn create_oob_invitation(&self) -> HarnessResult<String> {
        let invitation = self.aries_agent.out_of_band().create_invitation().await?;
        info!("Created out-of-band invitation: {}", invitation);
        Ok(json!({ "invitation": invitation, "state": "invitation-sent" }).to_string())
    }

    pub async fn receive_oob_invitation(&self, invitation: AriesMessage) -> HarnessResult<String> {
        info!("Received out-of-band invitation: {}", invitation);
        let id = self
            .aries_agent
            .out_of_band()
            .receive_invitation(invitation)?;
        Ok(json!({ "connection_id": id, "state": "invitation-received" }).to_string())
    }

    pub async fn get_oob(&self, id: &str) -> HarnessResult<String> {
        soft_assert_eq!(self.aries_agent.out_of_band().exists_by_id(id), true);
        Ok(json!({ "connection_id": id }).to_string())
    }
}

#[post("/send-invitation-message")]
async fn send_invitation_message(agent: web::Data<RwLock<HarnessAgent>>) -> impl Responder {
    agent.read().unwrap().create_oob_invitation().await
}

#[post("/receive-invitation")]
async fn receive_invitation_message(
    req: web::Json<Request<Option<AriesMessage>>>,
    agent: web::Data<RwLock<HarnessAgent>>,
) -> impl Responder {
    agent
        .read()
        .unwrap()
        .receive_oob_invitation(req.data.clone().ok_or_else(|| {
            HarnessError::from_msg(
                HarnessErrorType::InvalidJson,
                "Missing invitation in request body",
            )
        })?)
        .await
}

#[get("/{thread_id}")]
async fn get_oob(
    agent: web::Data<RwLock<HarnessAgent>>,
    path: web::Path<String>,
) -> impl Responder {
    agent.read().unwrap().get_oob(&path.into_inner()).await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command/out-of-band")
            .service(send_invitation_message)
            .service(receive_invitation_message),
    )
    .service(web::scope("/response/out-of-band").service(get_oob));
}
