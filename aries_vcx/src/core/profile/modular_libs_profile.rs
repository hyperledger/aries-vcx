use async_lock::RwLock;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_core::errors::error::VcxCoreResult;
use aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite, TaaConfigurator, TxnAuthrAgrmtOptions,
};
use aries_vcx_core::ledger::indy_vdr_ledger::{
    IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite, IndyVdrLedgerWriteConfig, ProtocolVersion,
};
use aries_vcx_core::ledger::request_signer::base_wallet::BaseWalletRequestSigner;
use aries_vcx_core::ledger::request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter, LedgerPoolConfig};
use aries_vcx_core::ledger::response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::ResponseParser;
use async_trait::async_trait;

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
    anoncreds_ledger_write: Arc<RwLock<dyn AnoncredsLedgerWrite + Send + Sync>>,
    indy_ledger_write: Arc<RwLock<dyn IndyLedgerWrite + Send + Sync>>,
    taa_configurator: Arc<RwLock<dyn TaaConfigurator + Send + Sync>>,
}

// The Proxy structs encapsulates dealing with RwLock while implementing IndyLedgerWrite trait,
// so that rest of the codebase can call trait methods below, without being concerned about obtaining
// read/write locks. This pattern also enabled us introduce mutability of ledger write structures
// (setting up TAA in runtime) without having to sync up codebase across the board due to newly
// introduced RwLock.
#[derive(Debug)]
pub struct IndyLedgerWriteProxy {
    indy_ledger_write: Arc<RwLock<dyn IndyLedgerWrite + Send + Sync>>,
}

#[derive(Debug)]
pub struct AnoncredsLedgerWriteProxy {
    anoncreds_ledger_write: Arc<RwLock<dyn AnoncredsLedgerWrite + Send + Sync>>,
}

#[async_trait]
impl IndyLedgerWrite for IndyLedgerWriteProxy {
    async fn publish_nym(
        &self,
        submitter_did: &str,
        target_did: &str,
        verkey: Option<&str>,
        data: Option<&str>,
        role: Option<&str>,
    ) -> VcxCoreResult<String> {
        self.indy_ledger_write
            .read()
            .await
            .publish_nym(submitter_did, target_did, verkey, data, role)
            .await
    }

    async fn set_endorser(&self, submitter_did: &str, request: &str, endorser: &str) -> VcxCoreResult<String> {
        self.indy_ledger_write
            .read()
            .await
            .set_endorser(submitter_did, request, endorser)
            .await
    }

    async fn endorse_transaction(&self, endorser_did: &str, request_json: &str) -> VcxCoreResult<()> {
        self.indy_ledger_write
            .read()
            .await
            .endorse_transaction(endorser_did, request_json)
            .await
    }

    async fn add_attr(&self, target_did: &str, attrib_json: &str) -> VcxCoreResult<String> {
        self.indy_ledger_write
            .read()
            .await
            .add_attr(target_did, attrib_json)
            .await
    }
}

#[async_trait]
impl AnoncredsLedgerWrite for AnoncredsLedgerWriteProxy {
    async fn publish_schema(
        &self,
        schema_json: &str,
        submitter_did: &str,
        endorser_did: Option<String>,
    ) -> VcxCoreResult<()> {
        self.anoncreds_ledger_write
            .read()
            .await
            .publish_schema(schema_json, submitter_did, endorser_did)
            .await
    }

    async fn publish_cred_def(&self, cred_def_json: &str, submitter_did: &str) -> VcxCoreResult<()> {
        self.anoncreds_ledger_write
            .read()
            .await
            .publish_cred_def(cred_def_json, submitter_did)
            .await
    }

    async fn publish_rev_reg_def(&self, rev_reg_def: &str, submitter_did: &str) -> VcxCoreResult<()> {
        self.anoncreds_ledger_write
            .read()
            .await
            .publish_rev_reg_def(rev_reg_def, submitter_did)
            .await
    }

    async fn publish_rev_reg_delta(
        &self,
        rev_reg_id: &str,
        rev_reg_entry_json: &str,
        submitter_did: &str,
    ) -> VcxCoreResult<()> {
        self.anoncreds_ledger_write
            .read()
            .await
            .publish_rev_reg_delta(rev_reg_id, rev_reg_entry_json, submitter_did)
            .await
    }
}

impl ModularLibsProfile {
    fn init_ledger_read(
        request_submitter: Arc<IndyVdrSubmitter>,
    ) -> VcxResult<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>> {
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
        Ok(IndyVdrLedgerRead::new(config_read))
    }

    fn init_ledger_write(
        wallet: Arc<dyn BaseWallet>,
        request_submitter: Arc<IndyVdrSubmitter>,
        taa_options: Option<TxnAuthrAgrmtOptions>,
    ) -> IndyVdrLedgerWrite<IndyVdrSubmitter, BaseWalletRequestSigner> {
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let config_write = IndyVdrLedgerWriteConfig {
            request_signer,
            request_submitter,
            taa_options,
            protocol_version: ProtocolVersion::node_1_4(),
        };
        IndyVdrLedgerWrite::new(config_write)
    }

    pub fn init(wallet: Arc<dyn BaseWallet>, ledger_pool_config: LedgerPoolConfig) -> VcxResult<Self> {
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(Arc::clone(&wallet)));

        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));

        let ledger_read = Self::init_ledger_read(request_submitter.clone())?;
        let ledger_write = Self::init_ledger_write(wallet.clone(), request_submitter, None);

        let ledger_read = Arc::new(ledger_read);
        let ledger_write = Arc::new(RwLock::new(ledger_write));
        Ok(ModularLibsProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger_read.clone(),
            indy_ledger_read: ledger_read.clone(),
            anoncreds_ledger_write: ledger_write.clone(),
            indy_ledger_write: ledger_write.clone(),
            taa_configurator: ledger_write.clone(),
        })
    }
}

#[async_trait]
impl Profile for ModularLibsProfile {
    fn inject_indy_ledger_read(self: Arc<Self>) -> Arc<dyn IndyLedgerRead> {
        Arc::clone(&self.indy_ledger_read)
    }

    fn inject_indy_ledger_write(self: Arc<Self>) -> Arc<dyn IndyLedgerWrite> {
        Arc::new(IndyLedgerWriteProxy {
            indy_ledger_write: Arc::clone(&self.indy_ledger_write),
        })
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        Arc::clone(&self.anoncreds)
    }

    fn inject_anoncreds_ledger_read(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerRead> {
        Arc::clone(&self.anoncreds_ledger_read)
    }

    fn inject_anoncreds_ledger_write(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerWrite> {
        Arc::new(AnoncredsLedgerWriteProxy {
            anoncreds_ledger_write: Arc::clone(&self.anoncreds_ledger_write),
        })
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::clone(&self.wallet)
    }

    async fn update_taa_configuration(self: Arc<Self>, taa_options: TxnAuthrAgrmtOptions) {
        self.taa_configurator
            .write()
            .await
            .set_txn_author_agreement_options(taa_options);
    }
}
