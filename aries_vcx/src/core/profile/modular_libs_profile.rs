use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::credx_anoncreds::IndyCredxAnonCreds,
    ledger::base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions},
    wallet::indy::IndySdkWallet,
};
use async_trait::async_trait;

use super::{
    ledger::{ArcIndyVdrLedgerRead, ArcIndyVdrLedgerWrite},
    Profile,
};
use crate::{
    core::profile::ledger::{build_ledger_components, VcxPoolConfig},
    errors::error::VcxResult,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModularLibsProfile {
    wallet: Arc<IndySdkWallet>,
    anoncreds: IndyCredxAnonCreds,
    indy_ledger_read: ArcIndyVdrLedgerRead,
    indy_ledger_write: ArcIndyVdrLedgerWrite,
}

impl ModularLibsProfile {
    pub fn init(wallet: Arc<IndySdkWallet>, vcx_pool_config: VcxPoolConfig) -> VcxResult<Self> {
        let anoncreds = IndyCredxAnonCreds::new(wallet.clone());
        let (ledger_read, ledger_write) = build_ledger_components(wallet.clone(), vcx_pool_config)?;

        Ok(ModularLibsProfile {
            wallet,
            anoncreds,
            indy_ledger_read: ledger_read,
            indy_ledger_write: ledger_write,
        })
    }
}

#[async_trait]
impl Profile for ModularLibsProfile {
    type LedgerRead = ArcIndyVdrLedgerRead;
    type LedgerWrite = ArcIndyVdrLedgerWrite;
    type Anoncreds = IndyCredxAnonCreds;
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

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        self.indy_ledger_write
            .set_txn_author_agreement_options(taa_options)
            .map_err(|e| e.into())
    }
}
