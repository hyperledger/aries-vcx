mod common;
use std::collections::VecDeque;

use mediator::aries_agent::client::transports::AriesReqwest;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::header::ACCEPT;

use crate::common::{prelude::*, test_setup::setup_env_logging};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

const ENDPOINT_ROOT: &str = "http://localhost:8005";

#[tokio::test]
async fn didcomm_connection_succeeds() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    let client = reqwest::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("register").unwrap();

    let oobi: OOBInvitation = client
        .get(endpoint_register)
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    info!(
        "Got invitation from register endpoint {}",
        serde_json::to_string_pretty(&oobi.clone()).unwrap()
    );
    let agent = mediator::aries_agent::AgentBuilder::new_demo_agent().await?;
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let _state = agent
        .establish_connection(oobi, &mut aries_transport)
        .await?;

    Ok(())
}
