use std::{env, sync::Arc};

use aries_vcx_core::{
    ledger::{
        indy_vdr_ledger::{
            IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite,
            IndyVdrLedgerWriteConfig, ProtocolVersion,
        },
        request_submitter::vdr_proxy::VdrProxySubmitter,
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    ResponseParser, VdrProxyClient,
};
use log::info;

use crate::devsetup::prepare_taa_options;

pub async fn dev_build_profile_vdr_proxy_ledger() -> (
    IndyVdrLedgerRead<VdrProxySubmitter, InMemoryResponseCacher>,
    IndyVdrLedgerWrite<VdrProxySubmitter>,
) {
    info!("dev_build_profile_vdr_proxy_ledger >>");

    let client_url =
        env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
    let client = VdrProxyClient::new(&client_url).unwrap();

    let request_submitter = VdrProxySubmitter::new(Arc::new(client));
    let response_parser = ResponseParser;
    let cacher_config = InMemoryResponseCacherConfig::builder()
        .ttl(std::time::Duration::from_secs(60))
        .capacity(1000)
        .unwrap()
        .build();
    let response_cacher = InMemoryResponseCacher::new(cacher_config);

    let config_read = IndyVdrLedgerReadConfig {
        request_submitter: request_submitter.clone(),
        response_parser,
        response_cacher,
        protocol_version: ProtocolVersion::Node1_4,
    };
    let ledger_read = IndyVdrLedgerRead::new(config_read);

    let config_write = IndyVdrLedgerWriteConfig {
        request_submitter,
        taa_options: prepare_taa_options(&ledger_read).await.unwrap(),
        protocol_version: ProtocolVersion::Node1_4,
    };
    let ledger_write = IndyVdrLedgerWrite::new(config_write);

    (ledger_read, ledger_write)
}
