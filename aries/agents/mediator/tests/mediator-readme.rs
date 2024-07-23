mod common;

use anyhow::Result;
use log::info;
use mediator::http_routes::ReadmeInfo;
use reqwest::header::ACCEPT;
use url::Url;

use crate::common::test_setup::setup_env_logging;

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

const ENDPOINT_ROOT: &str = "http://localhost:8005";

#[tokio::test]
async fn base_path_returns_readme() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);

    let client = reqwest::Client::new();
    let endpoint: Url = ENDPOINT_ROOT.parse().unwrap();

    let res = client
        .get(endpoint)
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?;
    info!("{:?}", res);

    let _: ReadmeInfo = res.json().await?;

    Ok(())
}
