use std::str::FromStr;

use indy_api_types::errors::prelude::*;
use cosmrs::rpc::endpoint::abci_query;
use cosmrs::tx::MsgType;
use log_derive::logfn;

use crate::domain::cheqd_ledger::prost_ext::ProstMessageExt;
use crate::domain::cheqd_ledger::cosmos_ext::CosmosMsgExt;
use crate::domain::cheqd_ledger::CheqdProtoBase;
use crate::domain::cheqd_ledger::bank::{MsgSend, Coin, MsgSendResponse, QueryBalanceRequest, QueryBalanceResponse};
use crate::services::CheqdLedgerService;

impl CheqdLedgerService {
    fn get_vector_coins_from_amount_and_denom(
        &self,
        amount: &str,
        denom: &str,
    ) -> IndyResult<Vec<Coin>> {
        let coin = Coin::new(denom.to_string(), amount.to_string());
        let mut coins = Vec::new();
        coins.push(coin);
        Ok(coins)
    }

    #[logfn(Info)]
    pub(crate) fn bank_build_msg_send(
        &self,
        from_address: &str,
        to_address: &str,
        amount: &str,
        denom: &str,
    ) -> IndyResult<Vec<u8>> {
        let coins: Vec<Coin> = self.get_vector_coins_from_amount_and_denom(amount, denom)?;
        let msg_send = MsgSend::new(
            from_address.to_string(),
            to_address.to_string(),
            coins,
        );
        msg_send
            .to_proto()?
            .to_msg()?
            .to_bytes()
    }

    #[logfn(Info)]
    pub(crate) fn bank_parse_msg_send_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        self.parse_msg_resp::<MsgSendResponse>(resp)
    }

    #[logfn(Info)]
    pub(crate) fn bank_build_query_balance(
        &self,
        address: String,
        denom: String,
    ) -> IndyResult<String> {
        let query_data = QueryBalanceRequest::new(address, denom);
        let path = format!("/cosmos.bank.v1beta1.Query/Balance");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;
        let req =
            abci_query::Request::new(Some(path), query_data.to_proto()?.to_bytes()?, None, true);
        json_string_result!(req)
    }

    #[logfn(Info)]
    pub(crate) fn bank_parse_query_balance_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        self.parse_query_resp::<QueryBalanceResponse>(resp)
    }
}
