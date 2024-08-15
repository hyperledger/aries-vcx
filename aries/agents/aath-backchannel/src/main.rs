#[allow(clippy::await_holding_lock)]
mod controllers;
mod error;
mod setup;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate aries_vcx_agent;
extern crate clap;
extern crate reqwest;
extern crate uuid;

use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

use actix_web::{middleware, web, App, HttpServer};
use aries_vcx_agent::{
    aries_vcx::{aries_vcx_wallet::wallet::indy::IndySdkWallet, messages::AriesMessage},
    Agent as AriesAgent,
};
use clap::Parser;
use controllers::out_of_band;

use crate::controllers::{
    connection, credential_definition, did_exchange, didcomm, general, issuance, presentation,
    revocation, schema,
};

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = "9020")]
    port: u32,
    #[clap(short, long, default_value = "false")]
    interactive: String,
}

#[derive(Copy, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum State {
    Initial,
    Invited,
    Requested,
    RequestSet,
    Responded,
    Complete,
    Failure,
    Unknown,
    ProposalSent,
    ProposalReceived,
    RequestReceived,
    CredentialSent,
    OfferReceived,
    RequestSent,
    PresentationSent,
    Done,
}

#[derive(Copy, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Active,
}

pub struct HarnessAgent {
    aries_agent: AriesAgent<IndySdkWallet>,
    status: Status,
    // did-exchange specific
    // todo: extra didx specific AATH service
    didx_msg_buffer: RwLock<Vec<AriesMessage>>,
    didx_pthid_to_thid: Mutex<HashMap<String, String>>,
    // A map of DIDExchange thread IDs to the intended recipient
    didx_thid_to_request_recipient_verkey: Mutex<HashMap<String, String>>,
}

#[macro_export]
macro_rules! soft_assert_eq {
    ($left:expr, $right:expr) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    return Err(HarnessError::from_msg(
                        HarnessErrorType::InternalServerError,
                        &format!(
                            r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#,
                            left_val, right_val
                        ),
                    ));
                }
            }
        }
    }};
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let opts: Opts = Opts::parse();

    let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());

    let aries_agent = setup::initialize(opts.port).await;

    info!("Starting aries back-channel on port {}", opts.port);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .app_data(web::Data::new(RwLock::new(HarnessAgent {
                aries_agent: aries_agent.clone(),
                status: Status::Active,
                didx_msg_buffer: Default::default(),
                didx_pthid_to_thid: Default::default(),
                didx_thid_to_request_recipient_verkey: Default::default(),
            })))
            .app_data(web::Data::new(RwLock::new(Vec::<AriesMessage>::new())))
            .service(
                web::scope("/agent")
                    .configure(connection::config)
                    .configure(did_exchange::config)
                    .configure(schema::config)
                    .configure(credential_definition::config)
                    .configure(issuance::config)
                    .configure(revocation::config)
                    .configure(presentation::config)
                    .configure(out_of_band::config)
                    .configure(general::config),
            )
            .service(web::scope("/didcomm").route("", web::post().to(didcomm::receive_message)))
    })
    .keep_alive(std::time::Duration::from_secs(30))
    .client_request_timeout(std::time::Duration::from_secs(30))
    .workers(1)
    .bind(format!("{}:{}", host, opts.port))?
    .run()
    .await
}
