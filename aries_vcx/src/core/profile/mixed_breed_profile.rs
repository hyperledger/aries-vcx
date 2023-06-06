use std::sync::Arc;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use aries_vcx_core::ledger::base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions};
use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_ledger::{IndySdkLedgerRead, IndySdkLedgerWrite},
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    PoolHandle, WalletHandle,
};
use async_trait::async_trait;

use super::profile::Profile;

#[derive(Debug)]
pub struct MixedBreedProfile {
    wallet: Arc<dyn BaseWallet>,
    anoncreds: Arc<dyn BaseAnonCreds>,

    // ledger reads
    anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    indy_ledger_read: Arc<dyn IndyLedgerRead>,

    // ledger writes
    anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    indy_ledger_write: Arc<dyn IndyLedgerWrite>,
}

impl MixedBreedProfile {
    pub fn new(indy_wallet_handle: WalletHandle, indy_pool_handle: PoolHandle) -> Self {
        let wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(indy_wallet_handle));
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));
        let ledger_read = Arc::new(IndySdkLedgerRead::new(indy_wallet_handle, indy_pool_handle));
        let ledger_write = Arc::new(IndySdkLedgerWrite::new(indy_wallet_handle, indy_pool_handle));

        MixedBreedProfile {
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
impl Profile for MixedBreedProfile {
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
        Arc::clone(&self.wallet)
    }

    fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            format!("update_taa_configuration no implemented for MixedBreedProfile"),
        ))
    }
}
