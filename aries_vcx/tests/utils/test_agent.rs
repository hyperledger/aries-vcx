use std::sync::Arc;

use aries_vcx::{
    core::profile::profile::Profile,
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
    utils::{
        constants::TRUSTEE_SEED,
        devsetup::{dev_build_featured_profile, dev_setup_wallet_indy},
        random::generate_random_seed,
    },
};
use aries_vcx_core::wallet::indy::IndySdkWallet;

pub struct TestAgent {
    pub profile: Arc<dyn Profile>,
    pub institution_did: String,
    pub genesis_file_path: String,
}

async fn create_test_agent_from_seed(seed: &str, genesis_file_path: String) -> TestAgent {
    let (institution_did, wallet_handle) = dev_setup_wallet_indy(seed).await;
    let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
    let profile = dev_build_featured_profile(genesis_file_path.clone(), wallet).await;
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();
    TestAgent {
        genesis_file_path,
        profile,
        institution_did,
    }
}

pub async fn create_test_agent_trustee(genesis_file_path: String) -> TestAgent {
    create_test_agent_from_seed(TRUSTEE_SEED, genesis_file_path).await
}

pub async fn create_test_agent(genesis_file_path: String) -> TestAgent {
    create_test_agent_from_seed(&generate_random_seed(), genesis_file_path).await
}
