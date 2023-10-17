use std::sync::Arc;

use aries_vcx::{
    global::settings::DEFAULT_LINK_SECRET_ALIAS, utils::devsetup::dev_build_profile_modular,
};
use aries_vcx_core::{
    self,
    anoncreds::base_anoncreds::BaseAnonCreds,
    wallet::indy::{
        wallet::{create_and_open_wallet, wallet_configure_issuer},
        IndySdkWallet, WalletConfig,
    },
};

use crate::{
    agent::{agent_config::AgentConfig, agent_struct::Agent},
    error::AgentResult,
    services::{
        connection::{ServiceConnections, ServiceEndpoint},
        credential_definition::ServiceCredentialDefinitions,
        holder::ServiceCredentialsHolder,
        issuer::ServiceCredentialsIssuer,
        prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries,
        schema::ServiceSchemas,
        verifier::ServiceVerifier,
    },
};

pub struct WalletInitConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
}

pub struct PoolInitConfig {
    pub genesis_path: String,
    pub pool_name: String,
}

pub struct InitConfig {
    pub enterprise_seed: String,
    pub pool_config: PoolInitConfig,
    pub wallet_config: WalletInitConfig,
    pub service_endpoint: ServiceEndpoint,
}

impl Agent {
    pub async fn initialize(init_config: InitConfig) -> AgentResult<Self> {
        let config_wallet = WalletConfig {
            wallet_name: init_config.wallet_config.wallet_name,
            wallet_key: init_config.wallet_config.wallet_key,
            wallet_key_derivation: init_config.wallet_config.wallet_kdf,
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };

        let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
        let config_issuer = wallet_configure_issuer(wallet_handle, &init_config.enterprise_seed)
            .await
            .unwrap();

        let wallet = Arc::new(IndySdkWallet::new(wallet_handle));

        let (ledger_read, ledger_write, anoncreds) =
            dev_build_profile_modular(init_config.pool_config.genesis_path);
        let ledger_read = Arc::new(ledger_read);
        let ledger_write = Arc::new(ledger_write);

        anoncreds
            .prover_create_link_secret(wallet.as_ref(), DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        let connections = Arc::new(ServiceConnections::new(
            ledger_read.clone(),
            wallet.clone(),
            init_config.service_endpoint,
        ));
        let schemas = Arc::new(ServiceSchemas::new(
            ledger_read.clone(),
            ledger_write.clone(),
            anoncreds,
            wallet.clone(),
            config_issuer.institution_did.clone(),
        ));
        let cred_defs = Arc::new(ServiceCredentialDefinitions::new(
            ledger_read.clone(),
            ledger_write.clone(),
            anoncreds,
            wallet.clone(),
        ));
        let rev_regs = Arc::new(ServiceRevocationRegistries::new(
            ledger_write.clone(),
            anoncreds,
            wallet.clone(),
            config_issuer.institution_did.clone(),
        ));
        let issuer = Arc::new(ServiceCredentialsIssuer::new(
            anoncreds,
            wallet.clone(),
            connections.clone(),
        ));
        let holder = Arc::new(ServiceCredentialsHolder::new(
            ledger_read.clone(),
            anoncreds,
            wallet.clone(),
            connections.clone(),
        ));
        let verifier = Arc::new(ServiceVerifier::new(
            ledger_read.clone(),
            anoncreds,
            wallet.clone(),
            connections.clone(),
        ));
        let prover = Arc::new(ServiceProver::new(
            ledger_read.clone(),
            anoncreds,
            wallet.clone(),
            connections.clone(),
        ));

        Ok(Self {
            ledger_read,
            ledger_write,
            anoncreds,
            wallet,
            connections,
            schemas,
            cred_defs,
            rev_regs,
            issuer,
            holder,
            verifier,
            prover,
            config: AgentConfig {
                config_wallet,
                config_issuer,
            },
        })
    }
}
