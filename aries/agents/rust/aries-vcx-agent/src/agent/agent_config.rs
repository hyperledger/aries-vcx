use aries_vcx_core::wallet::indy::{IssuerConfig, WalletConfig};
use display_as_json::Display;
use serde::Serialize;

#[derive(Clone, Serialize, Display)]
pub struct AgentConfig {
    pub config_wallet: WalletConfig,
    pub config_issuer: IssuerConfig,
}
