mod common;

use aries_vcx_wallet::wallet::askar::AskarWallet;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;

use crate::common::{prelude::*, test_setup::setup_env_logging};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

const ENDPOINT_ROOT: &str = "http://localhost:8005";

#[tokio::test]
async fn didcomm_connection_succeeds() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    let client = reqwest::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_invitation = base.join("invitation").unwrap();

    let oobi: OOBInvitation = client
        .get(endpoint_invitation)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    info!(
        "Got invitation {}",
        serde_json::to_string_pretty(&oobi.clone()).unwrap()
    );
    let agent = mediator::aries_agent::AgentBuilder::<AskarWallet>::new_demo_agent().await?;
    let mut aries_transport = reqwest::Client::new();
    let _state = agent
        .establish_connection(oobi, &mut aries_transport)
        .await?;

    Ok(())
}
