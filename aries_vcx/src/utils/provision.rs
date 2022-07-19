use indy_sys::WalletHandle;

use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};

use crate::error::prelude::*;
use crate::global::agency_client::get_agency_client;
use crate::global::settings;
use crate::libindy::utils::signus;

pub async fn provision_cloud_agent(wallet_handle: WalletHandle, provision_config: &AgentProvisionConfig) -> VcxResult<AgencyClientConfig> {
    let seed = provision_config.agent_seed.as_ref().map(String::as_str);
    let (my_did, my_vk) = signus::create_and_store_my_did(wallet_handle, seed, None).await?;
    let mut client = get_agency_client()?;
    client.provision_cloud_agent(
        &my_did, &my_vk,
        &provision_config.agency_did, &provision_config.agency_verkey, &provision_config.agency_endpoint,
    ).await?;
    let config = client.get_config()?;

    Ok(config)
}
