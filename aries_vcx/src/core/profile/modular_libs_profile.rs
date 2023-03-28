use std::sync::Arc;

use crate::errors::error::VcxResult;
use crate::plugins::{
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
    ledger::{
        base_ledger::BaseLedger,
        indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool},
    },
    wallet::base_wallet::BaseWallet,
};
use crate::plugins::ledger::indy_vdr_ledger::LedgerPoolConfig;

use super::profile::Profile;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModularLibsProfile {
    wallet: Arc<dyn BaseWallet>,
    ledger_pool: Arc<IndyVdrLedgerPool>,
}

impl ModularLibsProfile {
    pub fn new(wallet: Arc<dyn BaseWallet>, ledger_pool_config: LedgerPoolConfig) -> VcxResult<Self> {
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        Ok(ModularLibsProfile { wallet, ledger_pool })
    }
}

impl Profile for ModularLibsProfile {
    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time

        let ledger_pool = Arc::clone(&self.ledger_pool);
        let wallet = Arc::clone(&self.wallet);
        Arc::new(IndyVdrLedger::new(wallet, ledger_pool))
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time
        Arc::new(IndyCredxAnonCreds::new(self))
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::clone(&self.wallet)
    }
}
