use std::sync::Arc;

use super::profile::Profile;
use crate::{
    errors::error::VcxResult,
    plugins::{
        anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
        ledger::{
            base_ledger::BaseLedger,
            indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool},
        },
        wallet::base_wallet::BaseWallet,
    },
};

pub struct LedgerPoolConfig {
    pub genesis_file_path: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModularWalletProfile {
    wallet: Arc<dyn BaseWallet>,
    ledger_pool: Arc<IndyVdrLedgerPool>,
}

impl ModularWalletProfile {
    pub fn new(wallet: Arc<dyn BaseWallet>, ledger_pool_config: LedgerPoolConfig) -> VcxResult<Self> {
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        Ok(ModularWalletProfile { wallet, ledger_pool })
    }
}

impl Profile for ModularWalletProfile {
    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time

        let ledger_pool = Arc::clone(&self.ledger_pool);
        Arc::new(IndyVdrLedger::new(self, ledger_pool))
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::clone(&self.wallet)
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        // todo - in the future we should lazy eval and avoid creating a new instance each time
        Arc::new(IndyCredxAnonCreds::new(self))
    }
}
