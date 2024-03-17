use std::sync::Arc;

use aries_vcx::{
    common::ledger::{
        service_didsov::EndpointDidSov,
        transactions::{add_new_did, write_endpoint},
    },
    did_doc::schema::service::typed::ServiceType,
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
};
use aries_vcx_core::{
    self,
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
    ledger::indy_vdr_ledger::DefaultIndyLedgerRead,
    wallet::{
        base_wallet::{BaseWallet, ManageWallet},
        indy::{indy_wallet_config::IndyWalletConfig},
    },
};
use did_peer::resolver::PeerDidResolver;
use did_resolver_registry::ResolverRegistry;
use did_resolver_sov::resolution::DidSovResolver;
use display_as_json::Display;
use serde::Serialize;
use url::Url;
use aries_vcx::did_parser::Did;
use aries_vcx_core::ledger::indy_vdr_ledger::DefaultIndyLedgerWrite;
use aries_vcx_core::wallet::base_wallet::issuer_config::IssuerConfig;
use aries_vcx_core::wallet::indy::IndySdkWallet;

use crate::{
    agent::agent_struct::Agent,
    error::AgentResult,
    handlers::{
        connection::{ServiceConnections, ServiceEndpoint},
        credential_definition::ServiceCredentialDefinitions,
        did_exchange::DidcommHandlerDidExchange,
        holder::ServiceCredentialsHolder,
        issuer::ServiceCredentialsIssuer,
        out_of_band::ServiceOutOfBand,
        prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries,
        schema::ServiceSchemas,
        verifier::ServiceVerifier,
    },
};

#[derive(Serialize, Display)]
pub struct WalletInitConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
}

#[derive(Serialize, Display)]
pub struct InitConfig {
    pub service_endpoint: ServiceEndpoint,
}

impl <W: BaseWallet> Agent<W> {
    pub async fn build_indy_wallet(wallet_config: WalletInitConfig, isser_seed: String) -> (IndySdkWallet, IssuerConfig) {
        let config_wallet = IndyWalletConfig {
            wallet_name: wallet_config.wallet_name,
            wallet_key: wallet_config.wallet_key,
            wallet_key_derivation: wallet_config.wallet_kdf,
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        config_wallet.create_wallet().await.unwrap();
        let wallet = config_wallet.open_wallet().await.unwrap();
        let config_issuer = wallet
            .configure_issuer(&isser_seed)
            .await
            .unwrap();
        (wallet, config_issuer)
    }

    pub async fn setup_ledger (
        wallet: W,
        ledger_write: Arc<DefaultIndyLedgerWrite>,
        institution_did: &Did,
        service_endpoint: Url
    ) -> AgentResult<()> {
        let (public_did, _verkey) = add_new_did(
            &wallet,
            ledger_write.as_ref(),
            institution_did,
            None,
        )
            .await?;
        let endpoint = EndpointDidSov::create()
            .set_service_endpoint(service_endpoint)
            .set_types(Some(vec![ServiceType::DIDCommV1.to_string()]));
        write_endpoint(
            &wallet,
            ledger_write.as_ref(),
            &public_did,
            &endpoint,
        )
            .await?;
        Ok(())
    }

    pub async fn initialize(genesis_path: String, wallet: Arc<W>, service_endpoint: ServiceEndpoint, issuer_did: String) -> AgentResult<Agent<W>> {

        use aries_vcx_core::ledger::indy_vdr_ledger::{build_ledger_components, VcxPoolConfig};

        info!("dev_build_profile_modular >>");
        let vcx_pool_config = VcxPoolConfig {
            indy_vdr_config: None,
            response_cache_config: None,
            genesis_file_path: genesis_path,
        };

        let anoncreds = IndyCredxAnonCreds;
        let (ledger_read, ledger_write) = build_ledger_components(vcx_pool_config.clone()).unwrap();

        let ledger_read = Arc::new(ledger_read);
        let ledger_write = Arc::new(ledger_write);

        anoncreds
            .prover_create_link_secret(wallet.as_ref(), &DEFAULT_LINK_SECRET_ALIAS.to_string())
            .await
            .unwrap();

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
        let out_of_band = Arc::new(ServiceOutOfBand::new(
            wallet.clone(),
            service_endpoint,
        ));
        let schemas = Arc::new(ServiceSchemas::new(
            ledger_read.clone(),
            ledger_write.clone(),
            anoncreds,
            wallet.clone(),
            issuer_did.clone(),
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
            issuer_did.clone(),
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
            issuer_did,
        })
    }
}
