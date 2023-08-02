use std::{sync::Arc, time::Duration};

use crate::error::DidSovError;
use aries_vcx_core::{
    ledger::{
        indy_vdr_ledger::{IndyVdrLedgerRead, IndyVdrLedgerReadConfig, ProtocolVersion},
        request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    ResponseParser,
};

use super::ConcreteAttrReader;

// todo: make things work one way or the other
// impl TryFrom<LedgerPoolConfig> for ConcreteAttrReader {
//     type Error = DidSovError;
//
//     fn try_from(pool_config: LedgerPoolConfig) -> Result<Self, Self::Error> {
//         let ledger_pool = Arc::new(IndyVdrLedgerPool::new(pool_config)?);
//         let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));
//         let response_parser = Arc::new(ResponseParser::new());
//         let cacher_config = InMemoryResponseCacherConfig::builder()
//             .ttl(Duration::from_secs(60))
//             .capacity(1000)?
//             .build();
//         let response_cacher = Arc::new(InMemoryResponseCacher::new(cacher_config));
//         let config = IndyVdrLedgerReadConfig {
//             request_submitter,
//             response_parser,
//             response_cacher,
//             protocol_version: ProtocolVersion::node_1_4(),
//         };
//         let ledger = Arc::new(IndyVdrLedgerRead::new(config));
//         Ok(Self { ledger })
//     }
// }
