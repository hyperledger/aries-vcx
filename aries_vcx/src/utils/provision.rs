use std::sync::Arc;

use agency_client::{
    agency_client::AgencyClient,
    configuration::{AgencyClientConfig, AgentProvisionConfig},
};
use aries_vcx_core::wallet::{
    agency_client_wallet::ToBaseAgencyClientWallet, base_wallet::BaseWallet,
};

use crate::errors::error::prelude::*;

pub async fn provision_cloud_agent(
    client: &mut AgencyClient,
    wallet: Arc<impl BaseWallet + 'static>,
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
