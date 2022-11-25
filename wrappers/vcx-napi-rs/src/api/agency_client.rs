use napi_derive::napi;

use vcx::api_vcx::api_global::agency_client;
use vcx::aries_vcx::agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use vcx::aries_vcx::agency_client::messages::update_message::UIDsByConn;
use vcx::aries_vcx::agency_client::MessageStatusCode;
use vcx::errors::error::{LibvcxError, LibvcxErrorKind};
use vcx::serde_json;
use vcx::serde_json::json;

use crate::error::to_napi_err;

#[napi]
pub async fn update_webhook_url(webhook_url: String) -> napi::Result<()> {
    agency_client::update_webhook_url(&webhook_url)
        .await
        .map_err(to_napi_err)
}

#[napi]
pub fn create_agency_client_for_main_wallet(config: String) -> napi::Result<()> {
    let config = serde_json::from_str::<AgencyClientConfig>(&config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Deserialization error parsing config: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    agency_client::create_agency_client_for_main_wallet(&config).map_err(to_napi_err)?;
    Ok(())
}

#[napi]
pub async fn provision_cloud_agent(config: String) -> napi::Result<String> {
    let config = serde_json::from_str::<AgentProvisionConfig>(&config)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Deserialization error parsing config: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    let agency_client_config = agency_client::provision_cloud_agent(&config)
        .await
        .map_err(to_napi_err)?;
    Ok(json!(agency_client_config).to_string())
}

// todo: can we accept Vec<String> instead of Stringified JSON in place of uids_by_conns?
#[napi]
pub async fn messages_update_status(status_code: String, uids_by_conns: String) -> napi::Result<()> {
    let status_code: MessageStatusCode = serde_json::from_str(&format!("\"{}\"", status_code))
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Deserialization error parsing status_code: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    let uids_by_conns: Vec<UIDsByConn> = serde_json::from_str(&uids_by_conns)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!("Deserialization error parsing uids_by_conns: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;

    agency_client::agency_update_messages(status_code, uids_by_conns)
        .await
        .map_err(to_napi_err)?;
    Ok(())
}
