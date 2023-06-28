use std::sync::Arc;

use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx_core::{
    self,
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
    wallet::indy::{
        wallet::{create_and_open_wallet, wallet_configure_issuer},
        IndySdkWallet, WalletConfig,
    },
};

use crate::{
    agent::{agent_config::AgentConfig, agent_struct::Agent},
    error::AgentResult,
    services::{
        out_of_band::ServiceOutOfBand,
        connection::{ServiceConnections, ServiceEndpoint},
        did_exchange::ServiceDidExchange,
        schema::ServiceSchemas,
        credential_definition::ServiceCredentialDefinitions,
        revocation_registry::ServiceRevocationRegistries,
        holder::ServiceCredentialsHolder,
        issuer::ServiceCredentialsIssuer,
        prover::ServiceProver,
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

        use aries_vcx_core::ledger::indy_vdr_ledger::{build_ledger_components, VcxPoolConfig};

        info!("dev_build_profile_modular >>");
        let vcx_pool_config = VcxPoolConfig {
            indy_vdr_config: None,
            response_cache_config: None,
            genesis_file_path: init_config.pool_config.genesis_path,
        };

        let anoncreds = IndyCredxAnonCreds;
        let (ledger_read, ledger_write) = build_ledger_components(vcx_pool_config).unwrap();

        let ledger_read = Arc::new(ledger_read);
        let ledger_write = Arc::new(ledger_write);

        anoncreds
            .prover_create_link_secret(wallet.as_ref(), DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        // TODO: This setup should be easier
        // The default issuer did can't be used - its verkey is not in base58 - TODO: double-check
        let (public_did, _verkey) = add_new_did(
            &wallet,
            &profile.inject_indy_ledger_write(),
            &config_issuer.institution_did,
            None,
        )
            .await?;
        let endpoint = EndpointDidSov::create()
            .set_service_endpoint(init_config.service_endpoint.clone())
            .set_types(Some(vec![DidSovServiceType::DidCommunication]));
        write_endpoint(&profile.inject_indy_ledger_write(), &public_did, &endpoint).await?;

        let did_peer_resolver = PeerDidResolver::new();
        let did_sov_resolver =
            DidSovResolver::new(Arc::<ConcreteAttrReader>::new(profile.inject_indy_ledger_read().into()));
        let did_resolver_registry = Arc::new(
            ResolverRegistry::new()
                .register_resolver::<PeerDidResolver>("peer".into(), did_peer_resolver.into())
                .register_resolver::<DidSovResolver>("sov".into(), did_sov_resolver.into()),
        );

        let connections = Arc::new(ServiceConnections::new(
            ledger_read.clone(),
            wallet.clone(),
            init_config.service_endpoint.clone(),
        ));
        let did_exchange = Arc::new(ServiceDidExchange::new(
            Arc::clone(&profile),
            did_resolver_registry.clone(),
            init_config.service_endpoint.clone(),
            public_did,
        ));
        let out_of_band = Arc::new(ServiceOutOfBand::new(
            Arc::clone(&profile),
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
            did_exchange,
            out_of_band,
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
