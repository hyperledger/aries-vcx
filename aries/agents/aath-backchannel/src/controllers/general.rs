use crate::error::HarnessResult;
use crate::HarnessAgent;
use actix_web::{get, web, HttpResponse, Responder};
use std::sync::RwLock;

impl HarnessAgent {
    pub fn get_status_json(&self) -> HarnessResult<String> {
        Ok(json!({ "status": self.status }).to_string())
    }

    pub fn get_public_did(&self) -> HarnessResult<String> {
        let public_did = self.aries_agent.public_did();
        Ok(json!({ "did": format!("did:sov:{public_did}") }).to_string())
    }
}

#[get("/status")]
pub async fn get_status(agent: web::Data<RwLock<HarnessAgent>>) -> impl Responder {
    HttpResponse::Ok().body(agent.read().unwrap().get_status_json().unwrap())
}

#[get("/version")]
pub async fn get_version() -> impl Responder {
    HttpResponse::Ok().body("1.0.0")
}

#[get("/did")]
pub async fn get_public_did(agent: web::Data<RwLock<HarnessAgent>>) -> impl Responder {
    HttpResponse::Ok().body(agent.read().unwrap().get_public_did().unwrap())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/command")
            .service(get_status)
            .service(get_version)
            .service(get_public_did),
    );
}
