use std::sync::Arc;

use aries_vcx::core::profile::profile::Profile;
use aries_vcx::global::settings::DEFAULT_LINK_SECRET_ALIAS;
use aries_vcx::utils::devsetup::{dev_build_featured_profile, dev_setup_wallet_indy};
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::IndySdkWallet;

pub struct Alice {
    pub profile: Arc<dyn Profile>,
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
    async fn setup(profile: Arc<dyn Profile>, genesis_file_path: String) -> Alice {
        Alice {
            genesis_file_path,
            profile,
        }
    }
}
