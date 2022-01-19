use std::str::FromStr;

use indy_api_types::errors::prelude::*;
use cosmrs::rpc::endpoint::abci_query;

use crate::domain::cheqd_ledger::tx::{GetTxRequest, GetTxResponse, QuerySimulateRequest, QuerySimulateResponse};
use crate::domain::cheqd_ledger::CheqdProto;
use crate::services::CheqdLedgerService;

use log_derive::logfn;

impl CheqdLedgerService {
    #[logfn(Info)]
    pub(crate) fn build_query_get_tx_by_hash(
        &self,
        hash: &str,
    ) -> IndyResult<String> {
        let query_data = GetTxRequest::new(hash.to_string());
        let path = format!("/cosmos.tx.v1beta1.Service/GetTx");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;
        let req =
            abci_query::Request::new(Some(path), query_data.to_proto_bytes()?, None, true);
        json_string_result!(req)
    }

    #[logfn(Info)]
    pub(crate) fn cheqd_parse_query_get_tx_by_hash_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        self.parse_query_resp::<GetTxResponse>(resp)
    }

    #[logfn(Info)]
    pub(crate) fn tx_build_query_simulate(
        &self,
        tx_bytes: &[u8],
    ) -> IndyResult<String> {
        let query_data = QuerySimulateRequest::new(&tx_bytes.to_vec())?;

        let path = format!("/cosmos.tx.v1beta1.Service/Simulate");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;

        let req = abci_query::Request::new(
            Some(path),
            query_data.to_proto_bytes()?,
            None,
            true);

        json_string_result!(req)
    }

    #[logfn(Info)]
    pub(crate) fn tx_parse_query_simulate_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        self.parse_query_resp::<QuerySimulateResponse>(resp)
    }
}
