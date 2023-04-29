use std::sync::Arc;

use crate::error::DIDSovError;
use aries_vcx_core::{
    ledger::indy_vdr_ledger::{IndyVdrLedger, IndyVdrLedgerPool, LedgerPoolConfig},
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    INVALID_WALLET_HANDLE,
};

use super::ConcreteAttrReader;

impl TryFrom<LedgerPoolConfig> for ConcreteAttrReader {
    type Error = DIDSovError;

    fn try_from(pool_config: LedgerPoolConfig) -> Result<Self, Self::Error> {
        let wallet = Arc::new(IndySdkWallet::new(INVALID_WALLET_HANDLE)) as Arc<dyn BaseWallet>;
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(pool_config)?);
        Ok(Self {
            ledger: Arc::new(IndyVdrLedger::new(Arc::clone(&wallet), ledger_pool)),
        })
    }
}
