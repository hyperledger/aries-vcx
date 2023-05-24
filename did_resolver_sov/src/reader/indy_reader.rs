use std::sync::Arc;

use aries_vcx_core::{ledger::indy_ledger::IndySdkLedgerRead, PoolHandle, INVALID_WALLET_HANDLE};

use super::ConcreteAttrReader;

impl From<PoolHandle> for ConcreteAttrReader {
    fn from(pool_handle: PoolHandle) -> Self {
        Self {
            ledger: Arc::new(IndySdkLedgerRead::new(INVALID_WALLET_HANDLE, pool_handle)),
        }
    }
}
