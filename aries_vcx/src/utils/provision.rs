use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};

use crate::error::prelude::*;
use crate::libindy::utils::signus;
use crate::settings;
use crate::settings::get_agency_client;

pub async fn provision_cloud_agent(provision_config: &AgentProvisionConfig) -> VcxResult<AgencyClientConfig> {
    let (my_did, my_vk) = signus::main_wallet_create_and_store_my_did(provision_config.agent_seed.as_ref().map(String::as_str), None).await?;

    settings::get_agency_client_mut().unwrap().set_agency_did(&provision_config.agency_did);
    settings::get_agency_client_mut().unwrap().set_agency_vk(&provision_config.agency_verkey);
    settings::get_agency_client_mut().unwrap().set_agency_url(&provision_config.agency_endpoint);
    settings::get_agency_client_mut().unwrap().set_my_vk(&my_vk);
    settings::get_agency_client_mut().unwrap().set_my_pwdid(&my_did);
    settings::get_agency_client_mut().unwrap().set_agent_vk(&provision_config.agency_verkey); // This is reset when connection is established and agent did needs not be set before onboarding

    let mut client = get_agency_client()?;
    client.provision_cloud_agent(
        &my_did, &my_vk,
        &provision_config.agency_did, &provision_config.agency_verkey, &provision_config.agency_endpoint,
    ).await?;
    let config = client.get_config()?;
    Ok(config)
}
