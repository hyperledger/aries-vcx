use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{base_ledger::BaseLedger, indy_ledger::IndySdkLedger, vdr_proxy_ledger::VdrProxyLedger},
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    PoolHandle, VdrProxyClient, WalletHandle,
};

use crate::errors::error::VcxResult;

use super::profile::Profile;

#[derive(Debug)]
pub struct VdrProxyProfile {
    wallet: Arc<dyn BaseWallet>,
    ledger: Arc<dyn BaseLedger>,
    anoncreds: Arc<dyn BaseAnonCreds>,
}

impl VdrProxyProfile {
    pub fn new(wallet: Arc<dyn BaseWallet>, client: VdrProxyClient) -> Self {
        let ledger = Arc::new(VdrProxyLedger::new(wallet.clone(), client));
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));
        VdrProxyProfile {
            wallet,
            ledger,
            anoncreds,
        }
    }
}

impl Profile for VdrProxyProfile {
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
