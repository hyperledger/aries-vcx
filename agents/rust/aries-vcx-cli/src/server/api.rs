use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse};

use super::{handlers::handle_message, error::e500};
use crate::agent::CliAriesAgent;

pub async fn receive_message(
    req: web::Bytes,
    agent: web::Data<Arc<RwLock<CliAriesAgent>>>,
) -> Result<HttpResponse, actix_web::Error> {
    match agent.write() {
        Ok(agent_guard) => {
            handle_message(agent_guard, req.to_vec()).await.map_err(e500)?;
            Ok(HttpResponse::Ok().finish())
        }
        Err(err) => {
            error!("Failed to acquire read lock on agent: {:?}", err);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
