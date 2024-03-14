use aries_vcx_wallet::wallet::{
    base_wallet::issuer_config::IssuerConfig, indy::indy_wallet_config::IndyWalletConfig,
};
use display_as_json::Display;
use serde::Serialize;

#[derive(Clone, Serialize, Display)]
pub struct AgentConfig {
    pub config_wallet: IndyWalletConfig,
    pub config_issuer: IssuerConfig,
}
