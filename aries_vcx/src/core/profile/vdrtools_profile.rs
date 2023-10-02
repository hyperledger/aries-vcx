use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::indy_anoncreds::IndySdkAnonCreds, ledger::base_ledger::TxnAuthrAgrmtOptions,
    wallet::indy::IndySdkWallet,
};
use async_trait::async_trait;

use super::{
    ledger::{build_ledger_components, ArcIndyVdrLedgerRead, ArcIndyVdrLedgerWrite, VcxPoolConfig},
    profile::Profile,
};
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[derive(Debug)]
pub struct VdrtoolsProfile {
    wallet: Arc<IndySdkWallet>,
    anoncreds: IndySdkAnonCreds,
    indy_ledger_read: ArcIndyVdrLedgerRead,
    indy_ledger_write: ArcIndyVdrLedgerWrite,
}

impl VdrtoolsProfile {
    pub fn init(wallet: Arc<IndySdkWallet>, vcx_pool_config: VcxPoolConfig) -> VcxResult<Self> {
        let anoncreds = IndySdkAnonCreds::new(wallet.wallet_handle);
        let (ledger_read, ledger_write) = build_ledger_components(wallet.clone(), vcx_pool_config)?;

        Ok(VdrtoolsProfile {
            wallet,
            anoncreds,
            indy_ledger_read: ledger_read,
            indy_ledger_write: ledger_write,
        })
    }
}

#[async_trait]
impl Profile for VdrtoolsProfile {
    type LedgerRead = ArcIndyVdrLedgerRead;
    type LedgerWrite = ArcIndyVdrLedgerWrite;
    type Anoncreds = IndySdkAnonCreds;
    type Wallet = IndySdkWallet;

    fn ledger_read(&self) -> &Self::LedgerRead {
        &self.indy_ledger_read
    }

    fn ledger_write(&self) -> &Self::LedgerWrite {
        &self.indy_ledger_write
    }

    fn anoncreds(&self) -> &Self::Anoncreds {
        &self.anoncreds
    }

    fn wallet(&self) -> &Self::Wallet {
        &self.wallet
    }

    #[cfg(feature = "migration")]
    fn wallet_handle(&self) -> Option<aries_vcx_core::WalletHandle> {
        Some(self.wallet.wallet_handle)
    }

    fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            "update_taa_configuration no implemented for VdrtoolsProfile",
        ))
    }
}
