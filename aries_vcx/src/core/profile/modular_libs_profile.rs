use std::sync::Arc;
use std::time::Duration;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_core::ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite};
use aries_vcx_core::ledger::indy_vdr_ledger::{
    IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite, IndyVdrLedgerWriteConfig, ProtocolVersion,
    TxnAuthrAgrmtOptions,
};
use aries_vcx_core::ledger::request_signer::base_wallet::BaseWalletRequestSigner;
use aries_vcx_core::ledger::request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter, LedgerPoolConfig};
use aries_vcx_core::ledger::response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::ResponseParser;

use crate::errors::error::VcxResult;

use super::profile::Profile;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModularLibsProfile {
    pub(crate) wallet: Arc<dyn BaseWallet>,
    pub(crate) anoncreds: Arc<dyn BaseAnonCreds>,
    pub(crate) anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    pub(crate) anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    pub(crate) indy_ledger_read: Arc<dyn IndyLedgerRead>,
    pub(crate) indy_ledger_write: Arc<dyn IndyLedgerWrite>,
}

impl ModularLibsProfile {
    pub fn init_ledger_read(
        request_submitter: Arc<IndyVdrSubmitter>,
    ) -> VcxResult<Arc<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>>> {
        let response_parser = Arc::new(ResponseParser::new());
        let cacher_config = InMemoryResponseCacherConfig::builder()
            .ttl(Duration::from_secs(60))
            .capacity(1000)?
            .build();
        let response_cacher = Arc::new(InMemoryResponseCacher::new(cacher_config));

        let config_read = IndyVdrLedgerReadConfig {
            request_submitter: request_submitter.clone(),
            response_parser,
            response_cacher,
            protocol_version: ProtocolVersion::node_1_4(),
        };
        let ledger_read = Arc::new(IndyVdrLedgerRead::new(config_read));
        Ok(ledger_read)
    }

    pub fn init_ledger_write(
        wallet: Arc<dyn BaseWallet>,
        request_submitter: Arc<IndyVdrSubmitter>,
        taa_options: Option<TxnAuthrAgrmtOptions>,
    ) -> VcxResult<Arc<IndyVdrLedgerWrite<IndyVdrSubmitter, BaseWalletRequestSigner>>> {
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let config_write = IndyVdrLedgerWriteConfig {
            request_signer,
            request_submitter,
            taa_options,
            protocol_version: ProtocolVersion::node_1_4(),
        };
        let ledger_write = Arc::new(IndyVdrLedgerWrite::new(config_write));
        Ok(ledger_write)
    }

    pub fn init(wallet: Arc<dyn BaseWallet>, ledger_pool_config: LedgerPoolConfig) -> VcxResult<Self> {
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));

        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));

        let ledger_read = Self::init_ledger_read(request_submitter.clone())?;
        let ledger_write = Self::init_ledger_write(wallet.clone(), request_submitter, None)?;

        Ok(ModularLibsProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger_read.clone(),
            anoncreds_ledger_write: ledger_write.clone(),
            indy_ledger_read: ledger_read.clone(),
            indy_ledger_write: ledger_write,
        })
    }
}

impl Profile for ModularLibsProfile {
    fn inject_indy_ledger_read(self: Arc<Self>) -> Arc<dyn IndyLedgerRead> {
        Arc::clone(&self.indy_ledger_read)
    }

    fn inject_indy_ledger_write(self: Arc<Self>) -> Arc<dyn IndyLedgerWrite> {
        Arc::clone(&self.indy_ledger_write)
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        Arc::clone(&self.anoncreds)
    }

    fn inject_anoncreds_ledger_read(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerRead> {
        Arc::clone(&self.anoncreds_ledger_read)
    }

    fn inject_anoncreds_ledger_write(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerWrite> {
        Arc::clone(&self.anoncreds_ledger_write)
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::clone(&self.wallet)
    }
}
