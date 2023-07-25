use crate::errors::error::VcxResult;
use aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions;
use aries_vcx_core::ledger::indy_vdr_ledger::{
    IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite, IndyVdrLedgerWriteConfig, ProtocolVersion,
};
use aries_vcx_core::ledger::request_signer::base_wallet::BaseWalletRequestSigner;
use aries_vcx_core::ledger::request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter, LedgerPoolConfig};
use aries_vcx_core::ledger::response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::ResponseParser;
use std::sync::Arc;
use std::time::Duration;

pub fn build_ledger_components(
    wallet: Arc<dyn BaseWallet>,
    ledger_pool_config: LedgerPoolConfig,
) -> VcxResult<(
    Arc<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>>,
    Arc<IndyVdrLedgerWrite<IndyVdrSubmitter, BaseWalletRequestSigner>>,
)> {
    let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
    let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));

    let ledger_read = indyvdr_build_ledger_read(request_submitter.clone())?;
    let ledger_write = indyvdr_build_ledger_write(wallet, request_submitter, None);

    let ledger_read = Arc::new(ledger_read);
    let ledger_write = Arc::new(ledger_write);

    return Ok((ledger_read, ledger_write));
}

pub fn indyvdr_build_ledger_read(
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
