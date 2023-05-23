use std::sync::Arc;
use std::time::Duration;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_core::ledger::base_ledger::BaseLedger;
use aries_vcx_core::ledger::indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerConfig};
use aries_vcx_core::ledger::request_signer::base_wallet::BaseWalletRequestSigner;
use aries_vcx_core::ledger::request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter, LedgerPoolConfig};
use aries_vcx_core::ledger::response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::ResponseParser;

use crate::errors::error::VcxResult;

use super::profile::Profile;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModularLibsProfile {
    wallet: Arc<dyn BaseWallet>,
    ledger: Arc<dyn BaseLedger>,
    anoncreds: Arc<dyn BaseAnonCreds>,
}

impl ModularLibsProfile {
    pub fn new(wallet: Arc<dyn BaseWallet>, ledger_pool_config: LedgerPoolConfig) -> VcxResult<Self> {
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));
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
        Ok(ModularLibsProfile {
            wallet,
            ledger,
            anoncreds,
        })
    }
}

impl Profile for ModularLibsProfile {
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
