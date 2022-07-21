use aries_vcx::agency_client::error::AgencyClientResult;
use aries_vcx::agency_client::messages::update_message::UIDsByConn;
use aries_vcx::agency_client::MessageStatusCode;
use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::global::agency_client::get_main_agency_client;

pub async fn agency_update_messages(status_code: MessageStatusCode, uids_by_conns: Vec<UIDsByConn>) -> VcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_messages(status_code, uids_by_conns).await
        .map_err(|err| VcxError::from_msg(VcxErrorKind::UnknownError,
                                          format!("Error updating state of message in agency.\nError: {}", err)))
}

pub async fn agency_update_agent_webhook(webhook_url: &str) -> VcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_agent_webhook(webhook_url).await?;
    Ok(())
}