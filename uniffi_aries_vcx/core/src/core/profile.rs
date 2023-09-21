use std::sync::Arc;

use aries_vcx::{
    aries_vcx_core::wallet::indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
    core::profile::{profile::Profile, vdrtools_profile::VdrtoolsProfile},
    utils::mockdata::profile::mock_ledger::MockLedger,
};

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};

pub struct ProfileHolder {
    pub inner: Arc<dyn Profile>,
}

impl ProfileHolder {}

pub fn new_indy_profile(wallet_config: WalletConfig) -> VcxUniFFIResult<Arc<ProfileHolder>> {
    block_on(async {
        let wh = create_and_open_wallet(&wallet_config).await?;

        let wallet = IndySdkWallet::new(wh);
        let ledger = Arc::new(MockLedger);
        let profile = VdrtoolsProfile::init(
            Arc::new(wallet),
            ledger.clone(),
            ledger.clone(),
            ledger.clone(),
            ledger.clone(),
        );

        Ok(Arc::new(ProfileHolder {
            inner: Arc::new(profile),
        }))
    })
}
