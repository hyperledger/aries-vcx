use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::BaseLedger, wallet::base_wallet::BaseWallet,
};

pub trait Profile: std::fmt::Debug + Send + Sync {
    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger>;

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds>;

    fn inject_wallet(&self) -> Arc<dyn BaseWallet>;
}
