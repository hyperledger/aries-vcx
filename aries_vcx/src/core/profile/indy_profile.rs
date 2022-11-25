use std::sync::Arc;

use vdrtools::{WalletHandle, PoolHandle};

use crate::plugins::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{base_ledger::BaseLedger, indy_ledger::IndySdkLedger},
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
};

use super::profile::Profile;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct IndySdkProfile {
    pub indy_wallet_handle: WalletHandle,
    pub indy_pool_handle: PoolHandle
}

impl IndySdkProfile {
    pub fn new(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        IndySdkProfile { indy_wallet_handle, indy_pool_handle }
    }
}

impl Profile for IndySdkProfile {
    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time
        Arc::new(IndySdkLedger::new(self))
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time
        Arc::new(IndySdkWallet::new(self.indy_wallet_handle))
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time
        Arc::new(IndySdkAnonCreds::new(self))
    }
}
