use std::sync::{Arc, RwLock};

use actix_web::{dev::Server, middleware, web, App, HttpServer};

use crate::{agent::CliAriesAgent, configuration::AppConfig};

use super::api;

pub fn run_server(config: &AppConfig, agent: Arc<RwLock<CliAriesAgent>>) -> anyhow::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
            .app_data(web::Data::new(agent.clone()))
            .service(web::scope("/didcomm").route("", web::post().to(api::receive_message)))
    })
    .workers(1)
    .bind(format!("{}:{}", config.host(), config.port()))?
    .run())
}
