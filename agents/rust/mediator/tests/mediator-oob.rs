use anyhow::Result;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use url::Url;

const ENDPOINT_ROOT: &str = "http://localhost:8005";

#[test]
fn endpoint_register_json_returns_oob() -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register_json = base.join("/register.json").unwrap();

    let res = client
        .get(endpoint_register_json)
        .send()?
        .error_for_status()?;
    println!("{:?}", res);

    let _oob: OOBInvitation = res.json()?;

    Ok(())
}

#[test]
fn endpoint_register_returns_oob_with_correct_accept_header() -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("/register").unwrap();

    let res = client
        .get(endpoint_register)
        .header(ACCEPT, "application/json")
        .send()?
        .error_for_status()?;
    println!("{:?}", res);

    let _oob: OOBInvitation = res.json()?;

    Ok(())
}

#[test]
fn endpoint_register_returns_html_page() -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("/register").unwrap();

    let res = client.get(endpoint_register).send()?.error_for_status()?;
    println!("{:?}", res);

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
    let client = reqwest::blocking::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("/register").unwrap();

    let res = client.get(endpoint_register).send()?.error_for_status()?;
    println!("{:?}", res);

    let _html = res.text()?;
    // validate qr of html page
    unimplemented!("validate oob qr of returned html page");
}
