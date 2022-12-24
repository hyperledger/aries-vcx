use std::ops::Deref;
use std::sync::{RwLock, RwLockWriteGuard};

use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use crate::errors::mapping_from_ariesvcx::map_ariesvcx_result;
use aries_vcx::agency_client::agency_client::AgencyClient;
use aries_vcx::agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use aries_vcx::agency_client::messages::update_message::UIDsByConn;
use aries_vcx::agency_client::MessageStatusCode;
use aries_vcx::plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet;

use super::profile::get_main_wallet;

lazy_static! {
    pub static ref AGENCY_CLIENT: RwLock<AgencyClient> = RwLock::new(AgencyClient::new());
}

pub fn get_main_agency_client_mut() -> LibvcxResult<RwLockWriteGuard<'static, AgencyClient>> {
    let agency_client = AGENCY_CLIENT.write()?;
    Ok(agency_client)
}

pub fn get_main_agency_client() -> LibvcxResult<AgencyClient> {
    let agency_client = AGENCY_CLIENT.read()?.deref().clone();
    Ok(agency_client)
}

pub fn create_agency_client_for_main_wallet(config: &AgencyClientConfig) -> LibvcxResult<()> {
    let client = get_main_agency_client()?.configure(get_main_wallet().to_base_agency_client_wallet(), config)?;
    set_main_agency_client(client);
    Ok(())
}

pub fn reset_main_agency_client() {
    trace!("reset_agency_client >>>");
    let mut agency_client = AGENCY_CLIENT.write().unwrap();
    *agency_client = AgencyClient::new();
}

pub fn set_main_agency_client(new_agency_client: AgencyClient) {
    trace!("set_main_agency_client >>>");
    let mut agency_client = AGENCY_CLIENT.write().unwrap();
    *agency_client = new_agency_client;
}

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

pub async fn provision_cloud_agent(agency_config: &AgentProvisionConfig) -> LibvcxResult<AgencyClientConfig> {
    let wallet = get_main_wallet();
    let mut client = get_main_agency_client()?;
    let res = aries_vcx::utils::provision::provision_cloud_agent(&mut client, wallet, agency_config).await;
    map_ariesvcx_result(res)
}
