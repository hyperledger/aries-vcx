use aries_vcx_core::{
    ledger::{
        base_ledger::TxnAuthrAgrmtOptions,
        indy_vdr_ledger::{
            IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite,
            IndyVdrLedgerWriteConfig, ProtocolVersion,
        },
        request_submitter::vdr_ledger::IndyVdrSubmitter,
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    ResponseParser,
};

use crate::errors::error::VcxResult;

pub fn indyvdr_build_ledger_read(
    request_submitter: IndyVdrSubmitter,
    cache_config: InMemoryResponseCacherConfig,
) -> VcxResult<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>> {
    let response_parser = ResponseParser;
    let response_cacher = InMemoryResponseCacher::new(cache_config);

    let config_read = IndyVdrLedgerReadConfig {
        request_submitter,
        response_parser,
        response_cacher,
        protocol_version: ProtocolVersion::Node1_4,
    };
    Ok(IndyVdrLedgerRead::new(config_read))
}

pub fn indyvdr_build_ledger_write(
    request_submitter: IndyVdrSubmitter,
    taa_options: Option<TxnAuthrAgrmtOptions>,
) -> IndyVdrLedgerWrite<IndyVdrSubmitter> {
    let config_write = IndyVdrLedgerWriteConfig {
        request_submitter,
        taa_options,
        protocol_version: ProtocolVersion::Node1_4,
    };
    IndyVdrLedgerWrite::new(config_write)
}
