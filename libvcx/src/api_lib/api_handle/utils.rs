use aries_vcx::agency_client::messages::update_message::UIDsByConn;
use aries_vcx::agency_client::MessageStatusCode;

use crate::api_lib::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use crate::api_lib::global::agency_client::get_main_agency_client;

pub async fn agency_update_messages(
    status_code: MessageStatusCode,
    uids_by_conns: Vec<UIDsByConn>,
) -> LibvcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_messages(status_code, uids_by_conns).await.map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::UnknownError,
            format!("Error updating state of message in agency.\nError: {}", err),
        )
    })
}

pub async fn agency_update_agent_webhook(webhook_url: &str) -> LibvcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_agent_webhook(webhook_url).await?;
    Ok(())
}
