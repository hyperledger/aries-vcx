use std::{sync::Arc, time::Duration};

use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerConfig},
        request_signer::base_wallet::BaseWalletRequestSigner,
        request_submitter::vdr_proxy::VdrProxySubmitter,
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    ResponseParser, VdrProxyClient, WalletHandle,
};

use crate::errors::error::VcxResult;

use super::profile::Profile;

#[derive(Debug)]
pub struct VdrProxyProfile {
    wallet: Arc<dyn BaseWallet>,
    anoncreds: Arc<dyn BaseAnonCreds>,
    anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    indy_ledger_read: Arc<dyn IndyLedgerRead>,
    indy_ledger_write: Arc<dyn IndyLedgerWrite>,
}

impl VdrProxyProfile {
    pub fn new(wallet_handle: WalletHandle, client: VdrProxyClient) -> VcxResult<Self> {
        let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
        let anoncreds = Arc::new(IndySdkAnonCreds::new(wallet_handle));
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let request_submitter = Arc::new(VdrProxySubmitter::new(Arc::new(client)));
        let response_parser = Arc::new(ResponseParser::new());
        let cacher_config = InMemoryResponseCacherConfig::builder()
            .ttl(Duration::from_secs(60))
            .capacity(1000)?
            .build();
        let response_cacher = Arc::new(InMemoryResponseCacher::new(cacher_config));
        let config = IndyVdrLedgerConfig {
            request_signer,
            request_submitter,
            response_parser,
            response_cacher,
        };
        let ledger = Arc::new(IndyVdrLedger::new(config));
        Ok(VdrProxyProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger.clone(),
            anoncreds_ledger_write: ledger.clone(),
            indy_ledger_read: ledger.clone(),
            indy_ledger_write: ledger,
        })
    }
}

impl Profile for VdrProxyProfile {
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
