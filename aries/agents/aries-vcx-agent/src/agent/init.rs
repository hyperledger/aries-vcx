use std::sync::Arc;

use aries_vcx::{
    common::ledger::{
        service_didsov::EndpointDidSov,
        transactions::{add_new_did, write_endpoint},
    },
    did_doc::schema::service::typed::ServiceType,
    did_parser_nom::Did,
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
};
use aries_vcx_anoncreds::{
    self,
    anoncreds::{anoncreds::Anoncreds, base_anoncreds::BaseAnonCreds},
    errors::error::VcxAnoncredsError,
};
use aries_vcx_ledger::ledger::indy_vdr_ledger::{
    build_ledger_components, DefaultIndyLedgerRead, VcxPoolConfig,
};
use aries_vcx_wallet::wallet::{
    askar::{askar_wallet_config::AskarWalletConfig, key_method::KeyMethod, AskarWallet},
    base_wallet::{issuer_config::IssuerConfig, BaseWallet, ManageWallet},
};
use did_peer::resolver::PeerDidResolver;
use did_resolver_registry::ResolverRegistry;
use did_resolver_sov::resolution::DidSovResolver;
use display_as_json::Display;
use serde::Serialize;
use url::Url;
use uuid::Uuid;

use crate::{
    agent::agent_struct::Agent,
    error::AgentResult,
    handlers::{
        connection::ServiceConnections, credential_definition::ServiceCredentialDefinitions,
        did_exchange::DidcommHandlerDidExchange, holder::ServiceCredentialsHolder,
        issuer::ServiceCredentialsIssuer, out_of_band::ServiceOutOfBand, prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries, schema::ServiceSchemas,
        verifier::ServiceVerifier,
    },
};

#[derive(Serialize, Display)]
pub struct WalletInitConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
}

pub async fn build_askar_wallet(
    _wallet_config: WalletInitConfig,
    issuer_seed: String,
) -> (AskarWallet, IssuerConfig) {
    // TODO - use actual config with storage path etc
    // simple in-memory wallet
    let config_wallet = AskarWalletConfig::new(
        "sqlite://:memory:",
        KeyMethod::Unprotected,
        "",
        &Uuid::new_v4().to_string(),
    );
    let wallet = config_wallet.create_wallet().await.unwrap();
    let config_issuer = wallet.configure_issuer(&issuer_seed).await.unwrap();

    let anoncreds = Anoncreds;

    if let Err(err) = anoncreds
        .prover_create_link_secret(&wallet, &DEFAULT_LINK_SECRET_ALIAS.to_string())
        .await
    {
        match err {
            VcxAnoncredsError::DuplicationMasterSecret(_) => {} // ignore
            _ => panic!("{}", err),
        };
    }

    (wallet, config_issuer)
}

impl<W: BaseWallet> Agent<W> {
    pub async fn setup_ledger(
        genesis_path: String,
        wallet: Arc<W>,
        service_endpoint: Url,
        submitter_did: Did,
        create_new_issuer: bool,
    ) -> AgentResult<Did> {
        let vcx_pool_config = VcxPoolConfig {
            indy_vdr_config: None,
            response_cache_config: None,
            genesis_file_path: genesis_path,
        };
        let (_, ledger_write) = build_ledger_components(vcx_pool_config.clone()).unwrap();
        let public_did = match create_new_issuer {
            true => {
                add_new_did(wallet.as_ref(), &ledger_write, &submitter_did, None)
                    .await?
                    .0
            }
            false => submitter_did,
        };
        let endpoint = EndpointDidSov::create()
            .set_service_endpoint(service_endpoint.clone())
            .set_types(Some(vec![ServiceType::DIDCommV1.to_string()]));
        write_endpoint(wallet.as_ref(), &ledger_write, &public_did, &endpoint).await?;
        info!(
            "Agent::setup_ledger >> wrote data on ledger, public_did: {}, endpoint: {}",
            public_did, service_endpoint
        );
        Ok(public_did)
    }

    pub async fn initialize(
        genesis_path: String,
        wallet: Arc<W>,
        service_endpoint: Url,
        issuer_did: Did,
    ) -> AgentResult<Agent<W>> {
        info!("dev_build_profile_modular >>");
        let vcx_pool_config = VcxPoolConfig {
            indy_vdr_config: None,
            response_cache_config: None,
            genesis_file_path: genesis_path,
        };

        let anoncreds = Anoncreds;
        let (ledger_read, ledger_write) = build_ledger_components(vcx_pool_config.clone()).unwrap();

        let ledger_read = Arc::new(ledger_read);
        let ledger_write = Arc::new(ledger_write);

        let did_peer_resolver = PeerDidResolver::new();
        let did_sov_resolver: DidSovResolver<Arc<DefaultIndyLedgerRead>, DefaultIndyLedgerRead> =
            DidSovResolver::new(ledger_read.clone());
        let did_resolver_registry = Arc::new(
            ResolverRegistry::new()
                .register_resolver("peer".into(), did_peer_resolver)
                .register_resolver("sov".into(), did_sov_resolver),
        );

        let connections = Arc::new(ServiceConnections::new(
            ledger_read.clone(),
            wallet.clone(),
            service_endpoint.clone(),
        ));
        let did_exchange = Arc::new(DidcommHandlerDidExchange::new(
            wallet.clone(),
            did_resolver_registry,
            service_endpoint.clone(),
            issuer_did.to_string(),
        ));
        let out_of_band = Arc::new(ServiceOutOfBand::new(wallet.clone(), service_endpoint));
        let schemas = Arc::new(ServiceSchemas::new(
            ledger_read.clone(),
            ledger_write.clone(),
            anoncreds,
            wallet.clone(),
            issuer_did.to_string(),
        ));
        let cred_defs = Arc::new(ServiceCredentialDefinitions::new(
            ledger_read.clone(),
            ledger_write.clone(),
            anoncreds,
            wallet.clone(),
        ));
        let rev_regs = Arc::new(ServiceRevocationRegistries::new(
            ledger_write.clone(),
            ledger_read.clone(),
            anoncreds,
            wallet.clone(),
            issuer_did.to_string(),
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
            issuer_did: issuer_did.to_string(),
        })
    }
}
