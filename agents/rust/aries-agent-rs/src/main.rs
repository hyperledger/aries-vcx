mod api;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web, guard};
use crate::api::{provision_agent, get_agent_provision};

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Going to build HTTP Server");

    HttpServer::new(|| {
        App::new()
            .service(provision_agent)
            .service(get_agent_provision)
    })
        .bind("127.0.0.1:8806")?
        .run()
        .await
}