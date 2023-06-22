use std::sync::Arc;

use aries_vcx::{
    core::profile::{modular_libs_profile::ModularLibsProfile, profile::Profile},
    utils::{constants::GENESIS_PATH, devsetup::SetupProfile, get_temp_dir_path},
};
use aries_vcx_core::{
    ledger::request_submitter::vdr_ledger::LedgerPoolConfig,
    wallet::{agency_client_wallet::ToBaseAgencyClientWallet, base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    WalletHandle,
};
use async_trait::async_trait;
use cred_migrator::{Config, Credentials, KeyDerivationMethod, Locator};
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

async fn migrate_and_replace_profile(wallet_handle: WalletHandle) -> WalletHandle {
    let (credentials, config) = make_new_wallet_config();
    cred_migrator::migrate_wallet(
        wallet_handle,
        config.clone(),
        credentials.clone(),
        cred_migrator::vdrtools2credx::migrate_any_record,
    )
    .await
    .unwrap();

    Locator::instance()
        .wallet_controller
        .open(config.clone(), credentials.clone())
        .await
        .unwrap()
}

pub fn make_modular_profile(wallet_handle: WalletHandle) -> Arc<ModularLibsProfile> {
    let genesis_file_path = String::from(get_temp_dir_path(GENESIS_PATH).to_str().unwrap());
    let wallet = IndySdkWallet::new(wallet_handle);

    Arc::new(ModularLibsProfile::init(Arc::new(wallet), LedgerPoolConfig { genesis_file_path }).unwrap())
}

fn make_new_wallet_config() -> (Credentials, Config) {
    let wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_owned();
    let wallet_name = format!("wallet_{}", Uuid::new_v4());

    let credentials = Credentials {
        key: wallet_key,
        key_derivation_method: KeyDerivationMethod::RAW,
        rekey: None,
        rekey_derivation_method: KeyDerivationMethod::ARGON2I_MOD,
        storage_credentials: None,
    };

    let config = Config {
        id: wallet_name,
        storage_type: None,
        storage_config: None,
        cache: None,
    };

    (credentials, config)
}
