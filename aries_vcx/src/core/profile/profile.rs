use std::sync::Arc;

use crate::errors::error::VcxResult;
use aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
};

use async_trait::async_trait;

#[cfg(feature = "migration")]
use aries_vcx_core::WalletHandle;

#[async_trait]
pub trait Profile: std::fmt::Debug + Send + Sync {
    fn inject_indy_ledger_read(&self) -> Arc<dyn IndyLedgerRead>;

    fn inject_indy_ledger_write(&self) -> Arc<dyn IndyLedgerWrite>;

    fn inject_anoncreds(&self) -> Arc<dyn BaseAnonCreds>;

    fn inject_anoncreds_ledger_read(&self) -> Arc<dyn AnoncredsLedgerRead>;

    fn inject_anoncreds_ledger_write(&self) -> Arc<dyn AnoncredsLedgerWrite>;

    fn inject_wallet(&self) -> Arc<dyn BaseWallet>;

    #[cfg(feature = "migration")]
    fn wallet_handle(&self) -> Option<WalletHandle> {
        None
    }

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()>;
}
