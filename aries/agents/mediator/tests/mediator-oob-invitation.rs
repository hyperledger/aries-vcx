mod common;

use anyhow::Result;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use url::Url;

use crate::common::{prelude::*, test_setup::setup_env_logging};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

const ENDPOINT_ROOT: &str = "http://localhost:8005";

#[tokio::test]
async fn endpoint_invitation_returns_oob() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);

    let client = reqwest::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register_json = base.join("/invitation").unwrap();

    let res = client
        .get(endpoint_register_json)
        .send()
        .await?
        .error_for_status()?;
    info!("{:?}", res);

    let _oob: OOBInvitation = res.json().await?;

    Ok(())
}
