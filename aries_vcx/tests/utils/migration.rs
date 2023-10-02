use std::sync::Arc;

use aries_vcx::{
    core::profile::{modular_libs_profile::ModularLibsProfile, profile::Profile},
    global::settings::WALLET_KDF_RAW,
    utils::devsetup::{dev_build_profile_modular, SetupProfile},
};
use aries_vcx_core::{
    wallet::indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
    WalletHandle,
};
use async_trait::async_trait;
use uuid::Uuid;

use crate::utils::test_agent::TestAgent;

#[async_trait]
pub trait Migratable {
    type Output;

    async fn migrate(mut self) -> Self::Output;
}

#[async_trait]
impl<P> Migratable for SetupProfile<P>
where
    P: Profile,
{
    type Output = SetupProfile<ModularLibsProfile>;

    async fn migrate(mut self) -> Self::Output {
        info!("SetupProfile::migrate >>>");
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_to_new_wallet(old_wh).await;
        let wallet = Arc::new(IndySdkWallet::new(new_wh));
        let profile = dev_build_profile_modular(self.genesis_file_path.clone(), wallet);

        SetupProfile {
            institution_did: self.institution_did,
            profile,
            genesis_file_path: self.genesis_file_path,
        }
    }
}

#[async_trait]
impl<P> Migratable for TestAgent<P>
where
    P: Profile,
{
    type Output = TestAgent<ModularLibsProfile>;

    async fn migrate(mut self) -> Self::Output {
        info!("Faber::migrate >>>");
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_to_new_wallet(old_wh).await;
        let wallet = Arc::new(IndySdkWallet::new(new_wh));
        let profile = dev_build_profile_modular(self.genesis_file_path.clone(), wallet.clone());

        TestAgent {
            profile,
            institution_did: self.institution_did,
            genesis_file_path: self.genesis_file_path,
        }
    }
}

async fn migrate_to_new_wallet(src_wallet_handle: WalletHandle) -> WalletHandle {
    let wallet_config = make_wallet_config();
    let dest_wallet_handle = create_and_open_wallet(&wallet_config).await.unwrap();

    wallet_migrator::migrate_wallet(
        src_wallet_handle,
        dest_wallet_handle,
        wallet_migrator::vdrtools2credx::migrate_any_record,
    )
    .await
    .unwrap();

    dest_wallet_handle
}

fn make_wallet_config() -> WalletConfig {
    let wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_owned();
    let wallet_name = format!("wallet_{}", Uuid::new_v4());

    WalletConfig {
        wallet_name,
        wallet_key,
        wallet_key_derivation: WALLET_KDF_RAW.to_string(),
        ..Default::default()
    }
}
