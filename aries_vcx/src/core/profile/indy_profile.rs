use std::sync::Arc;

use vdrtools::{PoolHandle, WalletHandle};

use crate::plugins::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{base_ledger::BaseLedger, indy_ledger::IndySdkLedger},
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
};

use super::profile::Profile;

#[derive(Debug)]
pub struct IndySdkProfile {
    wallet: Arc<dyn BaseWallet>,
    ledger: Arc<dyn BaseLedger>,
    anoncreds: Arc<dyn BaseAnonCreds>,
}

impl IndySdkProfile {
    pub fn new(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        let wallet = Arc::new(IndySdkWallet::new(indy_wallet_handle));
        let ledger = Arc::new(IndySdkLedger::new(indy_wallet_handle, indy_pool_handle));
        let anoncreds = Arc::new(IndySdkAnonCreds::new(indy_wallet_handle, indy_pool_handle));
        IndySdkProfile {
            wallet,
            ledger,
            anoncreds,
        }
    }
}

impl Profile for IndySdkProfile {
    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger> {
        Arc::clone(&self.ledger)
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        Arc::clone(&self.anoncreds)
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::clone(&self.wallet)
    }
}
