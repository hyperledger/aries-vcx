use std::sync::Arc;

use super::profile::Profile;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use aries_vcx_core::ledger::base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions};
use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_ledger::{IndySdkLedgerRead, IndySdkLedgerWrite},
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    PoolHandle, WalletHandle,
};
use async_trait::async_trait;

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
    pub fn init(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        let wallet = Arc::new(IndySdkWallet::new(indy_wallet_handle));
        let anoncreds = Arc::new(IndySdkAnonCreds::new(indy_wallet_handle));
        let ledger_read = Arc::new(IndySdkLedgerRead::new(indy_wallet_handle, indy_pool_handle));
        let ledger_write = Arc::new(IndySdkLedgerWrite::new(indy_wallet_handle, indy_pool_handle));
        VdrtoolsProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger_read.clone(),
            anoncreds_ledger_write: ledger_write.clone(),
            indy_ledger_read: ledger_read,
            indy_ledger_write: ledger_write,
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
