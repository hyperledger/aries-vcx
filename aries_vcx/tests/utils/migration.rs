use std::sync::Arc;

use aries_vcx::{
    core::profile::modular_libs_profile::ModularLibsProfile,
    global::settings::WALLET_KDF_RAW,
    utils::{constants::GENESIS_PATH, devsetup::SetupProfile, get_temp_dir_path},
};
use aries_vcx_core::{
    indy::wallet::{create_and_open_wallet, WalletConfig},
    ledger::request_submitter::vdr_ledger::LedgerPoolConfig,
    wallet::{agency_client_wallet::ToBaseAgencyClientWallet, base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    WalletHandle,
};
use async_trait::async_trait;
use uuid::Uuid;

use super::devsetup_agent::test_utils::{Alice, Faber};

#[async_trait]
pub trait Migratable {
    async fn migrate(&mut self);
}

#[async_trait]
impl Migratable for SetupProfile {
    async fn migrate(&mut self) {
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_and_replace_profile(old_wh).await;
        self.profile = make_modular_profile(new_wh);
    }
}

#[async_trait]
impl Migratable for Alice {
    async fn migrate(&mut self) {
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_and_replace_profile(old_wh).await;
        self.profile = make_modular_profile(new_wh);
        let new_wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(new_wh));
        self.agency_client.wallet = new_wallet.to_base_agency_client_wallet();
    }
}

#[async_trait]
impl Migratable for Faber {
    async fn migrate(&mut self) {
        let old_wh = self.profile.wallet_handle().unwrap();
        let new_wh = migrate_and_replace_profile(old_wh).await;
        self.profile = make_modular_profile(new_wh);
        let new_wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(new_wh));
        self.agency_client.wallet = new_wallet.to_base_agency_client_wallet();
    }
}

async fn migrate_and_replace_profile(src_wallet_handle: WalletHandle) -> WalletHandle {
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

pub fn make_modular_profile(wallet_handle: WalletHandle) -> Arc<ModularLibsProfile> {
    let genesis_file_path = String::from(get_temp_dir_path(GENESIS_PATH).to_str().unwrap());
    let wallet = IndySdkWallet::new(wallet_handle);

    Arc::new(ModularLibsProfile::init(Arc::new(wallet), LedgerPoolConfig { genesis_file_path }).unwrap())
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
