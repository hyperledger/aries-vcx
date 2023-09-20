use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
        TxnAuthrAgrmtOptions,
    },
    wallet::{base_wallet::BaseWallet, indy::IndySdkWallet},
    WalletHandle,
};
use async_trait::async_trait;

use super::profile::Profile;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[derive(Debug)]
pub struct VdrtoolsProfile {
    wallet: Arc<IndySdkWallet>,
    anoncreds: Arc<dyn BaseAnonCreds>,
    anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    indy_ledger_read: Arc<dyn IndyLedgerRead>,
    indy_ledger_write: Arc<dyn IndyLedgerWrite>,
}

impl VdrtoolsProfile {
    pub fn init(
        wallet: Arc<IndySdkWallet>,
        anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
        anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
        indy_ledger_read: Arc<dyn IndyLedgerRead>,
        indy_ledger_write: Arc<dyn IndyLedgerWrite>,
    ) -> Self {
        let anoncreds = Arc::new(IndySdkAnonCreds::new(wallet.wallet_handle));
        VdrtoolsProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read,
            anoncreds_ledger_write,
            indy_ledger_read,
            indy_ledger_write,
        }
    }
}

#[async_trait]
impl Profile for VdrtoolsProfile {
    fn inject_indy_ledger_read(&self) -> Arc<dyn IndyLedgerRead> {
        Arc::clone(&self.indy_ledger_read)
    }

    fn inject_indy_ledger_write(&self) -> Arc<dyn IndyLedgerWrite> {
        Arc::clone(&self.indy_ledger_write)
    }

    fn inject_anoncreds(&self) -> Arc<dyn BaseAnonCreds> {
        Arc::clone(&self.anoncreds)
    }

    fn inject_anoncreds_ledger_read(&self) -> Arc<dyn AnoncredsLedgerRead> {
        Arc::clone(&self.anoncreds_ledger_read)
    }

    fn inject_anoncreds_ledger_write(&self) -> Arc<dyn AnoncredsLedgerWrite> {
        Arc::clone(&self.anoncreds_ledger_write)
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        self.wallet.clone()
    }

    #[cfg(feature = "migration")]
    fn wallet_handle(&self) -> Option<WalletHandle> {
        Some(self.wallet.wallet_handle)
    }

    fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            format!("update_taa_configuration no implemented for VdrtoolsProfile"),
        ))
    }
}
