use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_ledger::IndySdkLedger,
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    PoolHandle, WalletHandle,
};

use super::profile::Profile;

#[derive(Debug)]
pub struct VdrtoolsProfile {
    wallet: Arc<dyn BaseWallet>,
    anoncreds: Arc<dyn BaseAnonCreds>,
    anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    indy_ledger_read: Arc<dyn IndyLedgerRead>,
    indy_ledger_write: Arc<dyn IndyLedgerWrite>,
}

impl VdrtoolsProfile {
    pub fn new(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        let wallet = Arc::new(IndySdkWallet::new(indy_wallet_handle));
        let anoncreds = Arc::new(IndySdkAnonCreds::new(indy_wallet_handle));
        let ledger = Arc::new(IndySdkLedger::new(indy_wallet_handle, indy_pool_handle));
        VdrtoolsProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger.clone(),
            anoncreds_ledger_write: ledger.clone(),
            indy_ledger_read: ledger.clone(),
            indy_ledger_write: ledger,
        }
    }
}

impl Profile for VdrtoolsProfile {
    fn inject_indy_ledger_read(self: Arc<Self>) -> Arc<dyn IndyLedgerRead> {
        Arc::clone(&self.indy_ledger_read)
    }

    fn inject_indy_ledger_write(self: Arc<Self>) -> Arc<dyn IndyLedgerWrite> {
        Arc::clone(&self.indy_ledger_write)
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        Arc::clone(&self.anoncreds)
    }

    fn inject_anoncreds_ledger_read(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerRead> {
        Arc::clone(&self.anoncreds_ledger_read)
    }

    fn inject_anoncreds_ledger_write(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerWrite> {
        Arc::clone(&self.anoncreds_ledger_write)
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::clone(&self.wallet)
    }
}
