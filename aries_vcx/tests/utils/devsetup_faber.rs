use std::sync::Arc;

use aries_vcx::common::primitives::credential_schema::Schema;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::global::settings;
use aries_vcx::global::settings::{init_issuer_config, DEFAULT_LINK_SECRET_ALIAS};
use aries_vcx::utils::constants::TRUSTEE_SEED;
use aries_vcx::utils::devsetup::{dev_build_featured_profile, dev_setup_wallet_indy};
use aries_vcx::utils::random::generate_random_seed;
use aries_vcx_core::wallet::indy::IndySdkWallet;

pub struct Faber {
    pub profile: Arc<dyn Profile>,
    pub institution_did: String,
    pub schema: Schema,
    pub genesis_file_path: String,
}

async fn create_faber_from_seed(seed: &str, genesis_file_path: String) -> Faber {
    let (public_did, wallet_handle) = dev_setup_wallet_indy(seed).await;
    let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
    let profile = dev_build_featured_profile(genesis_file_path.clone(), wallet).await;
    profile
        .inject_anoncreds()
        .prover_create_link_secret(DEFAULT_LINK_SECRET_ALIAS)
        .await
        .unwrap();
    Faber::setup(profile, genesis_file_path, public_did).await
}

pub async fn create_faber_trustee(genesis_file_path: String) -> Faber {
    create_faber_from_seed(TRUSTEE_SEED, genesis_file_path).await
}

pub async fn create_faber(genesis_file_path: String) -> Faber {
    create_faber_from_seed(&generate_random_seed(), genesis_file_path).await
}

impl Faber {
    pub async fn setup(profile: Arc<dyn Profile>, genesis_file_path: String, institution_did: String) -> Faber {
        settings::reset_config_values_ariesvcx().unwrap();

        // todo: can delete following?
        init_issuer_config(&institution_did).unwrap();

        let faber = Faber {
            genesis_file_path,
            profile,
            institution_did,
            schema: Schema::default(),
        };
        faber
    }
}
