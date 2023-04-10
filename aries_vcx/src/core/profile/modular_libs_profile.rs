use std::sync::Arc;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_core::wallet::base_wallet::BaseWallet;

use crate::errors::error::VcxResult;
use crate::plugins::ledger::indy_vdr_ledger::LedgerPoolConfig;
use crate::plugins::ledger::{
    base_ledger::BaseLedger,
    indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool},
};

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
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        let ledger = Arc::new(IndyVdrLedger::new(Arc::clone(&wallet), ledger_pool));
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));
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
