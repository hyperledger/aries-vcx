//! Ledger service for Cosmos back-end

use async_std::sync::Arc;

use crate::services::{CheqdLedgerService, CheqdPoolService, CheqdKeysService, CryptoService};
use indy_wallet::WalletService;

mod cheqd;
mod auth;
mod bank;
mod tx;

pub(crate) struct CheqdLedgerController {
    cheqd_ledger_service: Arc<CheqdLedgerService>,
    cheqd_pool_service: Arc<CheqdPoolService>,
    cheqd_keys_service: Arc<CheqdKeysService>,
    crypto_service: Arc<CryptoService>,
    wallet_service: Arc<WalletService>
}

impl CheqdLedgerController {
    pub fn new(cheqd_ledger_service: Arc<CheqdLedgerService>,
               cheqd_pool_service: Arc<CheqdPoolService>,
               cheqd_keys_service: Arc<CheqdKeysService>,
               crypto_service: Arc<CryptoService>,
               wallet_service: Arc<WalletService>) -> Self {
        CheqdLedgerController {
            cheqd_ledger_service,
            cheqd_pool_service,
            cheqd_keys_service,
            crypto_service,
        wallet_service}
    }
}
