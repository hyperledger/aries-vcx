mod common;

use anyhow::Result;
use mediator_client::mediator_client::MediatorClient;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use url::Url;

use crate::common::{prelude::*, test_setup::setup_env_logging};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

const ENDPOINT_ROOT: &str = "http://localhost:8005";

#[tokio::test]
async fn endpoint_register_json_returns_oob() -> Result<()> {
    let client = MediatorClient::new(ENDPOINT_ROOT).unwrap();

    client.register().await.unwrap();

    Ok(())
}

#[test]
fn endpoint_register_returns_oob_with_correct_accept_header() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);

    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("/register").unwrap();

    let res = client
        .get(endpoint_register)
        .header(ACCEPT, "application/json")
        .send()?
        .error_for_status()?;
    info!("{:?}", res);

    let _oob: OOBInvitation = res.json()?;

    Ok(())
}

#[test]
fn endpoint_register_returns_html_page() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);

    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("/register").unwrap();

    let res = client.get(endpoint_register).send()?.error_for_status()?;
    info!("{:?}", res);

    assert!(res
        .headers()
        .get(CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .contains("text/html"));

    Ok(())
}

#[test]
#[ignore]
fn endpoint_register_returns_html_page_with_valid_oob_qr() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);

    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("/register").unwrap();

    let res = client.get(endpoint_register).send()?.error_for_status()?;
    info!("{:?}", res);

    let _html = res.text()?;
    // validate qr of html page
    unimplemented!("validate oob qr of returned html page");
}
