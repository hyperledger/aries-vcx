use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{
        base_ledger::BaseLedger, indy_vdr_ledger::IndyVdrLedger, request_submitter::vdr_proxy::VdrProxySubmitter,
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    VdrProxyClient, WalletHandle,
};

use super::profile::Profile;

#[derive(Debug)]
pub struct VdrProxyProfile {
    wallet: Arc<dyn BaseWallet>,
    ledger: Arc<dyn BaseLedger>,
    anoncreds: Arc<dyn BaseAnonCreds>,
}

impl VdrProxyProfile {
    pub fn new(wallet_handle: WalletHandle, client: VdrProxyClient) -> Self {
        let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
        let submitter = Arc::new(VdrProxySubmitter::new(Arc::new(client)));
        let ledger = Arc::new(IndyVdrLedger::new(wallet.clone(), submitter));
        let anoncreds = Arc::new(IndySdkAnonCreds::new(wallet_handle));
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
