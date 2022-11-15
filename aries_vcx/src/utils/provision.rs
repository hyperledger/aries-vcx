use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};

use crate::error::prelude::*;
use crate::plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet;
use crate::plugins::wallet::base_wallet::BaseWallet;

pub async fn provision_cloud_agent(
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
            &provision_config.agency_endpoint,
        )
        .await?;
    let config = client.get_config()?;

    Ok(config)
}
