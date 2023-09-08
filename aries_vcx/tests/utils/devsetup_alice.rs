use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::{AgencyClientConfig, AgentProvisionConfig};
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx::utils::devsetup::{
    dev_build_featured_profile, dev_setup_wallet_indy, AGENCY_DID, AGENCY_ENDPOINT, AGENCY_VERKEY,
};
use aries_vcx::utils::provision::provision_cloud_agent;
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::IndySdkWallet;

pub struct Alice {
    pub profile: Arc<dyn Profile>,
    pub config_agency: AgencyClientConfig,
    pub agency_client: AgencyClient,
    pub genesis_file_path: String,
}

pub async fn create_alice(genesis_file_path: String) -> Alice {
    let (_public_did, wallet_handle) = dev_setup_wallet_indy(&generate_random_seed()).await;
    let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
    let profile = dev_build_featured_profile(genesis_file_path.clone(), wallet).await;
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();
    Alice::setup(profile, genesis_file_path).await
}

impl Alice {
    pub async fn setup(profile: Arc<dyn Profile>, genesis_file_path: String) -> Alice {
        let config_provision_agent = AgentProvisionConfig {
            agency_did: AGENCY_DID.to_string(),
            agency_verkey: AGENCY_VERKEY.to_string(),
            agency_endpoint: AGENCY_ENDPOINT.parse().unwrap(),
            agent_seed: None,
        };
        let mut agency_client = AgencyClient::new();
        let config_agency = provision_cloud_agent(&mut agency_client, profile.inject_wallet(), &config_provision_agent)
            .await
            .unwrap();
        let alice = Alice {
            genesis_file_path,
            profile,
            agency_client,
            config_agency,
        };
        alice
    }
}
