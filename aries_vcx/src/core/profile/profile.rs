use std::sync::{Arc, RwLock};

use crate::errors::error::VcxResult;
use aries_vcx_core::ledger::base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
    wallet::base_wallet::BaseWallet,
};
use async_trait::async_trait;

#[async_trait]
pub trait Profile: std::fmt::Debug + Send + Sync {
    fn inject_indy_ledger_read(self: Arc<Self>) -> Arc<dyn IndyLedgerRead>;

    fn inject_indy_ledger_write(self: Arc<Self>) -> Arc<dyn IndyLedgerWrite>;

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds>;

    fn inject_anoncreds_ledger_read(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerRead>;

    fn inject_anoncreds_ledger_write(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerWrite>;

    fn inject_wallet(&self) -> Arc<dyn BaseWallet>;

    fn update_taa_configuration(self: Arc<Self>, taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()>;
}
