use aries_vcx::agency_client::configuration::AgencyClientConfig;
use aries_vcx_core::indy::wallet::{IssuerConfig, WalletConfig};

#[derive(Clone)]
pub struct AgentConfig {
    pub config_wallet: WalletConfig,
    pub config_agency_client: Option<AgencyClientConfig>,
    pub config_issuer: IssuerConfig,
}
