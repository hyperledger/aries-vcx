use std::{
    ops::Deref,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use aries_vcx::{
    agency_client::{
        agency_client::AgencyClient,
        configuration::{AgencyClientConfig, AgentProvisionConfig},
        messages::update_message::UIDsByConn,
        MessageStatusCode,
    },
    aries_vcx_core::wallet::{
        agency_client_wallet::ToBaseAgencyClientWallet, base_wallet::BaseWallet,
    },
    errors::error::VcxResult,
};

use super::profile::get_main_wallet;
use crate::errors::{
    error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
    mapping_from_ariesvcx::map_ariesvcx_result,
};

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
    let client = get_main_agency_client()?
        .configure(get_main_wallet()?.to_base_agency_client_wallet(), config)?;
    set_main_agency_client(client);
    Ok(())
}

pub fn reset_main_agency_client() {
    trace!("reset_agency_client >>>");
    let mut agency_client = AGENCY_CLIENT
        .write()
        .expect("Unable to access AGENCY_CLIENT");
    *agency_client = AgencyClient::new();
}

pub fn set_main_agency_client(new_agency_client: AgencyClient) {
    trace!("set_main_agency_client >>>");
    let mut agency_client = AGENCY_CLIENT
        .write()
        .expect("Unable to access AGENCY_CLIENT");
    *agency_client = new_agency_client;
}

pub async fn agency_update_messages(
    status_code: MessageStatusCode,
    uids_by_conns: Vec<UIDsByConn>,
) -> LibvcxResult<()> {
    let client = get_main_agency_client()?;
    client
        .update_messages(status_code, uids_by_conns)
        .await
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::UnknownError,
                format!("Error updating state of message in agency.\nError: {}", err),
            )
        })
}

pub async fn update_webhook_url(webhook_url: &str) -> LibvcxResult<()> {
    let client = get_main_agency_client()?;
    client.update_agent_webhook(webhook_url).await?;
    Ok(())
}

pub async fn provision_cloud_agent(
    agency_config: &AgentProvisionConfig,
) -> LibvcxResult<AgencyClientConfig> {
    let wallet = get_main_wallet()?;
    let mut client = get_main_agency_client()?;
    let res = provision_cloud_agent_inner(&mut client, wallet, agency_config).await;
    map_ariesvcx_result(res)
}

pub async fn provision_cloud_agent_inner(
    client: &mut AgencyClient,
    wallet: Arc<dyn BaseWallet>,
    provision_config: &AgentProvisionConfig,
) -> VcxResult<AgencyClientConfig> {
    let seed = provision_config.agent_seed.as_deref();
    let (my_did, my_vk) = wallet.create_and_store_my_did(seed, None).await?;
    client
        .provision_cloud_agent(
            wallet.to_base_agency_client_wallet(),
            &my_did,
            &my_vk,
            &provision_config.agency_did,
            &provision_config.agency_verkey,
            provision_config.agency_endpoint.clone(),
        )
        .await?;
    let config = client.get_config()?;

    Ok(config)
}

#[cfg(test)]
pub mod tests {
    use aries_vcx::{
        agency_client::{
            messages::update_message::UIDsByConn, testing::mocking::AgencyMockDecrypted,
            MessageStatusCode,
        },
        utils::{constants, devsetup::SetupMocks},
    };

    use crate::api_vcx::api_global::agency_client::{agency_update_messages, update_webhook_url};

    #[tokio::test]
    async fn test_update_institution_webhook() {
        let _setup = SetupMocks::init();
        update_webhook_url("https://example.com").await.unwrap();
    }

    #[tokio::test]
    async fn test_messages_update_status() {
        let _setup = SetupMocks::init();
        AgencyMockDecrypted::set_next_decrypted_response(
            constants::GET_MESSAGES_DECRYPTED_RESPONSE,
        );

        let uids_by_conns_str =
            String::from(r#"[{"pairwiseDID":"QSrw8hebcvQxiwBETmAaRs","uids":["mgrmngq"]}]"#);
        let uids_by_conns: Vec<UIDsByConn> = serde_json::from_str(&uids_by_conns_str).unwrap();
        agency_update_messages(MessageStatusCode::Received, uids_by_conns)
            .await
            .unwrap();
    }
}
