use std::{sync::Arc, time::Duration};

use crate::errors::error::VcxResult;
use aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions;
use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, indy_anoncreds::IndySdkAnonCreds},
    ledger::{
        base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
        indy_vdr_ledger::{
            IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite, IndyVdrLedgerWriteConfig, ProtocolVersion,
        },
        request_signer::base_wallet::BaseWalletRequestSigner,
        request_submitter::vdr_proxy::VdrProxySubmitter,
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    wallet::{base_wallet::BaseWallet, indy_wallet::IndySdkWallet},
    ResponseParser, VdrProxyClient, WalletHandle,
};
use async_trait::async_trait;

use super::{prepare_taa_options, profile::Profile};

#[derive(Debug)]
pub struct VdrProxyProfile {
    wallet: Arc<dyn BaseWallet>,
    anoncreds: Arc<dyn BaseAnonCreds>,
    anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead>,
    anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite>,
    indy_ledger_read: Arc<dyn IndyLedgerRead>,
    indy_ledger_write: Arc<dyn IndyLedgerWrite>,
}

impl VdrProxyProfile {
    pub async fn init(wallet_handle: WalletHandle, client: VdrProxyClient) -> VcxResult<Self> {
        let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
        let anoncreds = Arc::new(IndySdkAnonCreds::new(wallet_handle));
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let request_submitter = Arc::new(VdrProxySubmitter::new(Arc::new(client)));
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

        let config_write = IndyVdrLedgerWriteConfig {
            request_submitter,
            request_signer,
            taa_options: prepare_taa_options(ledger_read.clone()).await?,
            protocol_version: ProtocolVersion::node_1_4(),
        };
        let ledger_write = Arc::new(IndyVdrLedgerWrite::new(config_write));

        Ok(VdrProxyProfile {
            wallet,
            anoncreds,
            anoncreds_ledger_read: ledger_read.clone(),
            anoncreds_ledger_write: ledger_write.clone(),
            indy_ledger_read: ledger_read,
            indy_ledger_write: ledger_write,
        })
    }
}

#[async_trait]
impl Profile for VdrProxyProfile {
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

    async fn update_taa_configuration(self: Arc<Self>, taa_options: TxnAuthrAgrmtOptions) {
        todo!()
    }
}
