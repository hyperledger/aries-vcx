use agency_client::agent_utils;

use crate::error::prelude::*;
use crate::libindy::utils::signus;
use crate::settings;

#[derive(Serialize, Deserialize, Default, Builder, Clone)]
#[builder(setter(into, strip_option), default)]
pub struct AgentProvisionConfig {
    pub agency_did: String,
    pub agency_verkey: String,
    pub agency_endpoint: String,
    pub agent_seed: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgencyClientConfig {
    pub agency_did: String,
    pub agency_endpoint: String,
    pub agency_verkey: String,
    pub remote_to_sdk_did: String,
    pub remote_to_sdk_verkey: String,
    pub sdk_to_remote_did: String,
    pub sdk_to_remote_verkey: String,
}

pub async fn provision_cloud_agent(provision_agent_config: &AgentProvisionConfig) -> VcxResult<AgencyClientConfig> {
    let (my_did, my_vk) = signus::create_and_store_my_did(provision_agent_config.agent_seed.as_ref().map(String::as_str), None).await?;

    settings::get_agency_client_mut().unwrap().set_agency_did(&provision_agent_config.agency_did);
    settings::get_agency_client_mut().unwrap().set_agency_vk(&provision_agent_config.agency_verkey);
    settings::get_agency_client_mut().unwrap().set_agency_url(&provision_agent_config.agency_endpoint);
    settings::get_agency_client_mut().unwrap().set_my_vk(&my_vk);
    settings::get_agency_client_mut().unwrap().set_my_pwdid(&my_did);
    settings::get_agency_client_mut().unwrap().set_agent_vk(&provision_agent_config.agency_verkey); // This is reset when connection is established and agent did needs not be set before onboarding

    let (agent_did, agent_vk) = agent_utils::onboarding(&my_did, &my_vk, &provision_agent_config.agency_did).await?;

    Ok(AgencyClientConfig {
        agency_did: provision_agent_config.agency_did.clone(),
        agency_endpoint: provision_agent_config.agency_endpoint.clone(),
        agency_verkey: provision_agent_config.agency_verkey.clone(),
        remote_to_sdk_did: agent_did,
        remote_to_sdk_verkey: agent_vk,
        sdk_to_remote_did: my_did,
        sdk_to_remote_verkey: my_vk,
    })
}
