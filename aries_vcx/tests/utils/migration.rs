use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use aries_vcx::utils::devsetup::make_modular_profile;
use aries_vcx::{
    core::profile::modular_libs_profile::ModularLibsProfile, global::settings::WALLET_KDF_RAW,
    utils::devsetup::SetupProfile,
};
use aries_vcx_core::{
    ledger::request_submitter::vdr_ledger::LedgerPoolConfig,
    wallet::base_wallet::BaseWallet,
    WalletHandle,
};
use aries_vcx_core::wallet::indy::agency_client_wallet::ToBaseAgencyClientWallet;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use aries_vcx_core::wallet::indy::wallet::create_and_open_wallet;
use aries_vcx_core::wallet::indy::WalletConfig;

use crate::utils::devsetup_alice::Alice;
use crate::utils::devsetup_faber::Faber;

#[async_trait]
pub trait Migratable {
    async fn migrate(&mut self);
}

#[async_trait]
impl Migratable for SetupProfile {
    async fn migrate(&mut self) {
        info!("SetupProfile::migrate >>>");
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_to_new_wallet(old_wh).await;
        self.profile = make_modular_profile(new_wh, self.genesis_file_path.clone());
    }
}

#[async_trait]
impl Migratable for Alice {
    async fn migrate(&mut self) {
        info!("Alice::migrate >>>");
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_to_new_wallet(old_wh).await;
        self.profile = make_modular_profile(new_wh, self.genesis_file_path.clone());
        let new_wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(new_wh));
        self.agency_client.wallet = new_wallet.to_base_agency_client_wallet();
    }
}

#[async_trait]
impl Migratable for Faber {
    async fn migrate(&mut self) {
        info!("Faber::migrate >>>");
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_to_new_wallet(old_wh).await;
        self.profile = make_modular_profile(new_wh, self.genesis_file_path.clone());
        let new_wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(new_wh));
        self.agency_client.wallet = new_wallet.to_base_agency_client_wallet();
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
