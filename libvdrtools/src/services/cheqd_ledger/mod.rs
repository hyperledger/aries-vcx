//! Ledger service for Cheqd back-end

use cosmrs::proto::cosmos::base::abci::v1beta1::TxMsgData;
use cosmrs::rpc::endpoint::broadcast::tx_commit::Response;
use cosmrs::rpc::endpoint::abci_query::Response as QueryResponse;
use indy_api_types::errors::prelude::*;
use log_derive::logfn;

use crate::domain::cheqd_ledger::prost_ext::ProstMessageExt;
use crate::domain::cheqd_ledger::CheqdProto;

use serde::Serialize;

mod auth;
mod cheqd;
mod bank;
mod tx;

pub(crate) struct CheqdLedgerService {}

impl CheqdLedgerService {
    pub(crate) fn new() -> Self {
        Self {}
    }

    #[logfn(Info)]
    fn parse_msg_resp<R: Serialize>(&self, resp: &str) -> IndyResult<String>
        where
            R: CheqdProto,
    {
        let resp: Response = serde_json::from_str(resp)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Cannot deserialize Response. Err: {:?}", err),
            ))?;
        let data = resp.deliver_tx.data.as_ref().ok_or(IndyError::from_msg(
            IndyErrorKind::InvalidState,
            "Expected response data but got None",
        ))?;
        let data = data.value();
        let tx_msg = TxMsgData::from_bytes(&data)?;
        let result = R::from_proto_bytes(&tx_msg.data[0].data)?;
        json_string_result!(result)
    }

    #[logfn(Info)]
    pub(crate) fn parse_query_resp<R: CheqdProto + Serialize>(&self, resp: &str) -> IndyResult<String> {
        let resp: QueryResponse = serde_json::from_str(resp)
            .map_err(|err| err_msg(
                IndyErrorKind::InvalidStructure,
                format!("Cannot deserialize QueryResponse. Err: {:?}", err),
            ))?;
        let result = R::from_proto_bytes(&resp.response.value)?;
        json_string_result!(result)
    }
}
