use std::sync::Arc;

use async_trait::async_trait;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite, TaaConfigurator, TxnAuthrAgrmtOptions,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;

use crate::core::profile::ledger::{build_ledger_components, VcxPoolConfig};
use crate::errors::error::VcxResult;

use super::profile::Profile;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModularLibsProfile {
    wallet: Arc<dyn BaseWallet>,
    anoncreds: Arc<dyn BaseAnonCreds>,

    // ledger reads
    anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    indy_ledger_read: Arc<dyn IndyLedgerRead>,

    // ledger writes
    anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    indy_ledger_write: Arc<dyn IndyLedgerWrite>,
    taa_configurator: Arc<dyn TaaConfigurator>,
}

impl ModularLibsProfile {
    pub fn init(wallet: Arc<dyn BaseWallet>, vcx_pool_config: VcxPoolConfig) -> VcxResult<Self> {
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));
        let (ledger_read, ledger_write) = build_ledger_components(wallet.clone(), vcx_pool_config)?;

        Ok(ModularLibsProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger_read.clone(),
            indy_ledger_read: ledger_read,
            anoncreds_ledger_write: ledger_write.clone(),
            indy_ledger_write: ledger_write.clone(),
            taa_configurator: ledger_write,
        })
    }
}

#[async_trait]
impl Profile for ModularLibsProfile {
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

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()> {
        self.taa_configurator
            .set_txn_author_agreement_options(taa_options)
            .map_err(|e| e.into())
    }
}
