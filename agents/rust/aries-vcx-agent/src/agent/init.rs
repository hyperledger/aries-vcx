use std::sync::Arc;

use aries_vcx::{
    agency_client::{agency_client::AgencyClient, configuration::AgentProvisionConfig},
    global::settings::init_issuer_config,
    indy::{
        ledger::pool::{create_pool_ledger_config, open_pool_ledger, PoolConfigBuilder},
        wallet::{create_wallet_with_master_secret, open_wallet, wallet_configure_issuer, WalletConfig},
    },
    utils::provision::provision_cloud_agent, core::profile::{indy_profile::IndySdkProfile, profile::Profile},
};

use crate::{
    agent::{agent_config::AgentConfig, agent_struct::Agent},
    error::AgentResult,
    services::{
        connection::{ServiceConnections, ServiceEndpoint},
        credential_definition::ServiceCredentialDefinitions,
        holder::ServiceCredentialsHolder,
        issuer::ServiceCredentialsIssuer,
        mediated_connection::ServiceMediatedConnections,
        prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries,
        schema::ServiceSchemas,
        verifier::ServiceVerifier,
    },
};

pub struct AgencyInitConfig {
    pub agency_endpoint: String,
    pub agency_did: String,
    pub agency_verkey: String,
}

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
    pub agency_config: Option<AgencyInitConfig>,
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

        create_wallet_with_master_secret(&config_wallet).await.unwrap();
        let wallet_handle = open_wallet(&config_wallet).await.unwrap();

        let config_issuer = wallet_configure_issuer(wallet_handle, &init_config.enterprise_seed)
            .await
            .unwrap();
        init_issuer_config(&config_issuer).unwrap();

        let pool_config = PoolConfigBuilder::default()
            .genesis_path(&init_config.pool_config.genesis_path)
            .build()
            .expect("Failed to build pool config");
        create_pool_ledger_config(
            &init_config.pool_config.pool_name,
            &init_config.pool_config.genesis_path,
        )
        .await
        .unwrap();
        let pool_handle = open_pool_ledger(&init_config.pool_config.pool_name, Some(pool_config))
            .await
            .unwrap();

        let indy_profile = IndySdkProfile::new(wallet_handle, pool_handle);
        let profile: Arc<dyn Profile> = Arc::new(indy_profile);
        let wallet = profile.inject_wallet();

        let (mediated_connections, config_agency_client) = if let Some(agency_config) = init_config.agency_config {
            let config_provision_agent = AgentProvisionConfig {
                agency_did: agency_config.agency_did,
                agency_verkey: agency_config.agency_verkey,
                agency_endpoint: agency_config.agency_endpoint,
                agent_seed: None,
            };
            let mut agency_client = AgencyClient::new();
            let config_agency_client =
                provision_cloud_agent(&mut agency_client, wallet, &config_provision_agent)
                    .await
                    .unwrap();
            (
                Some(Arc::new(ServiceMediatedConnections::new(
                    Arc::clone(&profile),
                    config_agency_client.clone(),
                ))),
                Some(config_agency_client),
            )
        } else {
            (None, None)
        };

        let connections = Arc::new(ServiceConnections::new(
            Arc::clone(&profile),
            init_config.service_endpoint,
        ));
        let schemas = Arc::new(ServiceSchemas::new(
            Arc::clone(&profile),
            config_issuer.institution_did.clone(),
        ));
        let cred_defs = Arc::new(ServiceCredentialDefinitions::new(Arc::clone(&profile)));
        let rev_regs = Arc::new(ServiceRevocationRegistries::new(
            Arc::clone(&profile),
            config_issuer.institution_did.clone(),
        ));
        let issuer = Arc::new(ServiceCredentialsIssuer::new(Arc::clone(&profile), connections.clone()));
        let holder = Arc::new(ServiceCredentialsHolder::new(
            Arc::clone(&profile),
            connections.clone(),
        ));
        let verifier = Arc::new(ServiceVerifier::new(Arc::clone(&profile), connections.clone()));
        let prover = Arc::new(ServiceProver::new(Arc::clone(&profile), connections.clone()));

        Ok(Self {
            profile,
            connections,
            mediated_connections,
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
                config_agency_client,
            },
        })
    }
}
