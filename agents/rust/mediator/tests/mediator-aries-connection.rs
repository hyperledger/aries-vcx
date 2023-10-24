mod common;
use std::collections::VecDeque;

use common::{prelude::*, test_setup::OneTimeInit};
use mediator::aries_agent::transports::AriesReqwest;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::header::ACCEPT;

const ENDPOINT_ROOT: &str = "http://localhost:8005";

struct TestSetupAries;
impl OneTimeInit for TestSetupAries {
    fn one_time_setup_code(&self) {
        fn setup_logging() {
            let env = env_logger::Env::default().default_filter_or("info");
            env_logger::init_from_env(env);
        }
        fn load_dot_env() {
            let _ = dotenvy::dotenv();
        }
        load_dot_env();
        setup_logging();
    }
}

#[tokio::test]
async fn didcomm_connection_succeeds() -> Result<()> {
    TestSetupAries.init();
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
    info!("Got invitation from register endpoint {:?}", oobi);
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
