use aries_vcx::agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};

use crate::api_lib::errors::error::LibvcxResult;
use crate::api_lib::errors::mapping_from_ariesvcx::map_ariesvcx_result;
use crate::api_lib::global::agency_client::get_main_agency_client;
use crate::api_lib::global::profile::get_main_wallet;

pub async fn provision_cloud_agent(agency_config: &AgentProvisionConfig) -> LibvcxResult<AgencyClientConfig> {
    let wallet = get_main_wallet();
    let mut client = get_main_agency_client()?;
    let res = aries_vcx::utils::provision::provision_cloud_agent(&mut client, wallet, agency_config).await;
    map_ariesvcx_result(res)
}
