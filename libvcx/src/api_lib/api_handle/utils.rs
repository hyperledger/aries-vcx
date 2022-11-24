use aries_vcx::agency_client::messages::update_message::UIDsByConn;
use aries_vcx::agency_client::MessageStatusCode;
use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use napi_derive::napi;

use crate::api_lib::global::agency_client::get_main_agency_client;
use crate::api_lib::utils::logger::LibvcxDefaultLogger;

pub async fn agency_update_messages(status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> VcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_messages(status_code, uids_by_conns).await.map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::UnknownError,
            format!("Error updating state of message in agency.\nError: {}", err),
        )
    })
}

pub async fn agency_update_agent_webhook(webhook_url: &str) -> VcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_agent_webhook(webhook_url).await?;
    Ok(())
}

#[napi]
pub fn init_default_logger(pattern: Option<String>) -> ::napi::Result<()> {
    LibvcxDefaultLogger::init(pattern)
        .map_err(|err| err.into())
}
