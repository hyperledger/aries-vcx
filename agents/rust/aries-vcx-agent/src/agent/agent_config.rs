use aries_vcx_core::wallet::indy::{IssuerConfig, WalletConfig};

#[derive(Clone)]
pub struct AgentConfig {
    pub config_wallet: WalletConfig,
    pub config_issuer: IssuerConfig,
}
