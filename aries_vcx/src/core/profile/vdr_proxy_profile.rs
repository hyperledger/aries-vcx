use std::{sync::Arc, time::Duration};

use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
    ledger::{
        base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions},
        indy_vdr_ledger::{
            IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite,
            IndyVdrLedgerWriteConfig, ProtocolVersion,
        },
        request_signer::base_wallet::BaseWalletRequestSigner,
        request_submitter::vdr_proxy::VdrProxySubmitter,
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    wallet::indy::IndySdkWallet,
    ResponseParser, VdrProxyClient,
};
use async_trait::async_trait;

use super::{prepare_taa_options, Profile};
use crate::errors::error::VcxResult;

#[derive(Debug)]
pub struct VdrProxyProfile {
    wallet: Arc<IndySdkWallet>,
    anoncreds: IndySdkAnonCreds,
    indy_ledger_read: Arc<IndyVdrLedgerRead<VdrProxySubmitter, InMemoryResponseCacher>>,
    indy_ledger_write: IndyVdrLedgerWrite<VdrProxySubmitter, BaseWalletRequestSigner>,
}

impl VdrProxyProfile {
    pub async fn init(wallet: Arc<IndySdkWallet>, client: VdrProxyClient) -> VcxResult<Self> {
        let anoncreds = Arc::new(IndyCredxAnonCreds::new(wallet.clone()));
        let request_signer = Arc::new(BaseWalletRequestSigner::new(wallet.clone()));
        let request_submitter = Arc::new(VdrProxySubmitter::new(Arc::new(client)));
        let response_parser = Arc::new(ResponseParser);
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
        let ledger_write = IndyVdrLedgerWrite::new(config_write);

        Ok(VdrProxyProfile {
            wallet,
            anoncreds,
            indy_ledger_read: ledger_read,
            indy_ledger_write: ledger_write,
        })
    }
}

#[async_trait]
impl Profile for VdrProxyProfile {
    type LedgerRead = IndyVdrLedgerRead<VdrProxySubmitter, InMemoryResponseCacher>;
    type LedgerWrite = IndyVdrLedgerWrite<VdrProxySubmitter, BaseWalletRequestSigner>;
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
        self.ledger_write()
            .set_txn_author_agreement_options(taa_options)
            .map_err(|e| e.into())
    }
}
