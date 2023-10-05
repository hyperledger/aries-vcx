use std::{sync::Arc, time::Duration};

use aries_vcx_core::{
    ledger::{
        base_ledger::TxnAuthrAgrmtOptions,
        indy_vdr_ledger::{
            IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite,
            IndyVdrLedgerWriteConfig, ProtocolVersion,
        },
        request_signer::base_wallet::BaseWalletRequestSigner,
        request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    wallet::base_wallet::BaseWallet,
    PoolConfig, ResponseParser,
};

use crate::errors::error::VcxResult;
/// TODO: Rename these
pub type ArcIndyVdrLedgerRead = IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>;
pub type ArcIndyVdrLedgerWrite = IndyVdrLedgerWrite<IndyVdrSubmitter, BaseWalletRequestSigner>;

pub struct VcxPoolConfig {
    pub genesis_file_path: String,
    pub indy_vdr_config: Option<PoolConfig>,
    pub response_cache_config: Option<InMemoryResponseCacherConfig>,
}

pub fn build_ledger_components(
    wallet: Arc<dyn BaseWallet>,
    pool_config: VcxPoolConfig,
) -> VcxResult<(ArcIndyVdrLedgerRead, ArcIndyVdrLedgerWrite)> {
    let indy_vdr_config = match pool_config.indy_vdr_config {
        None => PoolConfig::default(),
        Some(cfg) => cfg,
    };
    let cache_config = match pool_config.response_cache_config {
        None => InMemoryResponseCacherConfig::builder()
            .ttl(Duration::from_secs(60))
            .capacity(1000)?
            .build(),
        Some(cfg) => cfg,
    };

    let ledger_pool =
        IndyVdrLedgerPool::new(pool_config.genesis_file_path, indy_vdr_config, vec![])?;

    let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));

    let ledger_read = indyvdr_build_ledger_read(request_submitter.clone(), cache_config)?;
    let ledger_write = indyvdr_build_ledger_write(wallet, request_submitter, None);

    Ok((ledger_read, ledger_write))
}

pub fn indyvdr_build_ledger_read(
    request_submitter: Arc<IndyVdrSubmitter>,
    cache_config: InMemoryResponseCacherConfig,
) -> VcxResult<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>> {
    let response_parser = Arc::new(ResponseParser);
    let response_cacher = Arc::new(InMemoryResponseCacher::new(cache_config));

    let config_read = IndyVdrLedgerReadConfig {
        request_submitter,
        response_parser,
        response_cacher,
        protocol_version: ProtocolVersion::node_1_4(),
    };
    Ok(IndyVdrLedgerRead::new(config_read))
}

pub fn indyvdr_build_ledger_write(
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
