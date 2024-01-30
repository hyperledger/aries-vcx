use aries_vcx_core::wallet::{
    base_wallet::issuer_config::IssuerConfig, indy::wallet_config::WalletConfig,
};

#[derive(Clone)]
pub struct AgentConfig {
    pub config_wallet: WalletConfig,
    pub config_issuer: IssuerConfig,
}
