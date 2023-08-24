use std::sync::Arc;

use async_trait::async_trait;

use aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite},
    wallet::base_wallet::BaseWallet,
    WalletHandle,
};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

use super::profile::Profile;

#[derive(Debug)]
pub struct VdrtoolsProfile<A, R, W>
where
    A: BaseAnonCreds,
    R: IndyLedgerRead,
    W: IndyLedgerWrite,
{
    wallet: Arc<IndySdkWallet>,
    anoncreds: A,
    indy_ledger_read: R,
    indy_ledger_write: W,
}

impl<A, R, W> VdrtoolsProfile<A, R, W>
where
    A: BaseAnonCreds,
    R: IndyLedgerRead,
    W: IndyLedgerWrite,
{
    pub fn init(wallet: Arc<IndySdkWallet>, indy_ledger_read: R, indy_ledger_write: W) -> Self {
        let anoncreds = Arc::new(IndySdkAnonCreds::new(wallet.wallet_handle));
        VdrtoolsProfile {
            wallet,
            anoncreds,
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
