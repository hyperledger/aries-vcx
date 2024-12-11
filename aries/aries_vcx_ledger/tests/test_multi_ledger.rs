#[cfg(feature = "cheqd")]
mod test_cheqd {
    use std::sync::Arc;

    use aries_vcx_ledger::{
        errors::error::VcxLedgerResult,
        ledger::{
            cheqd::CheqdAnoncredsLedgerRead,
            indy_vdr_ledger::{IndyVdrLedgerRead, IndyVdrLedgerReadConfig},
            multi_ledger::MultiLedgerAnoncredsRead,
            request_submitter::RequestSubmitter,
            response_cacher::noop::NoopResponseCacher,
        },
    };
    use async_trait::async_trait;
    use did_cheqd::resolution::resolver::DidCheqdResolver;
    use indy_vdr::pool::ProtocolVersion;
    use mockall::mock;

    mock! {
        pub RequestSubmitter {}
        #[async_trait]
        impl RequestSubmitter for RequestSubmitter {
            async fn submit(&self, request: indy_vdr::pool::PreparedRequest) -> VcxLedgerResult<String>;
        }
    }

    fn dummy_indy_vdr_reader() -> IndyVdrLedgerRead<MockRequestSubmitter, NoopResponseCacher> {
        IndyVdrLedgerRead::new(IndyVdrLedgerReadConfig {
            request_submitter: MockRequestSubmitter::new(),
            response_parser: indy_ledger_response_parser::ResponseParser,
            response_cacher: NoopResponseCacher,
            protocol_version: ProtocolVersion::Node1_4,
        })
    }

    // asserts the successful construction using our defined anoncreds ledger readers.
    #[test]
    fn test_construction() {
        let cheqd =
            CheqdAnoncredsLedgerRead::new(Arc::new(DidCheqdResolver::new(Default::default())));
        let indy = dummy_indy_vdr_reader();

        let _multi_ledger = MultiLedgerAnoncredsRead::new()
            .register_reader(cheqd)
            .register_reader(indy);
    }
}
