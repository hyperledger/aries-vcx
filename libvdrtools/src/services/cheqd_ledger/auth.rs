use std::convert::TryInto;
use std::str::FromStr;

use cosmrs::{Coin, tx, AccountId};
use cosmrs::crypto::PublicKey;
use cosmrs::rpc::endpoint::abci_query;
use cosmrs::tendermint::block::Height;
use cosmrs::tx::{AuthInfo, Fee, Msg, SignDoc, SignerInfo, Raw};
use cosmrs::proto::cosmos::tx::v1beta1::TxRaw;
use cosmrs::rpc::endpoint::abci_query::Response as QueryResponse;
use indy_api_types::errors::prelude::*;
use crate::domain::cheqd_ledger::auth::{QueryAccountResponse, Account};
use crate::domain::cheqd_ledger::CheqdProto;
use crate::domain::cheqd_ledger::cosmos_ext::{
    CosmosMsgExt,
    CosmosSignDocExt,
};
use crate::domain::cheqd_ledger::cheqd::v1::messages::{
    MsgWriteRequest,
    MsgWriteRequestPayload,
};
use crate::services::CheqdLedgerService;
use crate::utils::cheqd_crypto::check_proofs;
use crate::utils::cheqd_ledger::make_verification_id;

impl CheqdLedgerService {
    pub(crate) async fn auth_build_tx(
        &self,
        chain_id: &str,
        sender_public_key: &str,
        msg: &[u8],
        account_number: u64,
        sequence_number: u64,
        max_gas: u64,
        max_coin_amount: u64,
        max_coin_denom: &str,
        account_id: &str,
        timeout_height: u64,
        memo: &str,
    ) -> IndyResult<(SignDoc, Vec<u8>)> {
        let msg = Msg::from_bytes(msg)?;

        let timeout_height: Height = timeout_height.try_into()?;

        let tx_body = tx::Body::new(vec![msg], memo, timeout_height);

        let signer_info = Self::build_signer_info(sender_public_key, sequence_number)?;

        let auth_info =
            Self::build_auth_info(max_gas, max_coin_amount, max_coin_denom, account_id, signer_info)?;

        let chain_id = chain_id.try_into()?;

        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)?;
        let sign_doc_bytes = sign_doc.to_bytes()?;
        Ok((sign_doc, sign_doc_bytes))
    }

    fn build_auth_info(
        max_gas: u64,
        max_coin: u64,
        max_coin_denom: &str,
        account_id: &str,
        signer_info: SignerInfo,
    ) -> IndyResult<AuthInfo> {
        let amount = Coin {
            denom: max_coin_denom.parse()?,
            amount: max_coin.into(),
        };
        let account_id = AccountId::from_str(account_id)?;
        let fee = Fee {
            amount: vec![amount],
            gas_limit: max_gas.into(),
            payer: Some(account_id),
            granter: None,
        };
        let auth_info = signer_info.auth_info(fee);
        Ok(auth_info)
    }

    fn build_signer_info(public_key: &str, sequence_number: u64) -> IndyResult<SignerInfo> {
        let public_key = PublicKey::from_json(public_key)?;
        let public_key: PublicKey = public_key.into();

        let signer_info = SignerInfo::single_direct(Some(public_key), sequence_number);
        Ok(signer_info)
    }

    pub(crate) fn auth_build_query_account(
        &self,
        address: &str,
    ) -> IndyResult<String> {
        // let mut encoded_path = 0x01.to_bytes()?;
        // encoded_path.push_str(address);
        let mut query_data = vec!(0x01_u8);
        let acc = AccountId::from_str(address)?;
        query_data.append(acc.to_bytes().to_vec().as_mut());
        let path = format!("/store/acc/key");
        let path = cosmrs::tendermint::abci::Path::from_str(&path)?;
        let req = abci_query::Request::new(Some(path), query_data, None, true);
        json_string_result!(req)
    }

    pub(crate) fn auth_parse_query_account_resp(
        &self,
        resp: &str,
    ) -> IndyResult<String> {
        let resp: QueryResponse = serde_json::from_str(resp).to_indy(
            IndyErrorKind::InvalidStructure,
            "Cannot deserialize Response for QueryAccount request",
        )?;
        check_proofs(&resp)?;

        if resp.response.value.is_empty() {
            // ToDo: after adding method for decoding key to account_id in response,
            // info about absent account should be added here.
            return Err(IndyError::from(
                IndyErrorKind::QueryAccountDoesNotExist));
        }

        let account = Account::from_proto_bytes(&resp.response.value)?;
        let response = QueryAccountResponse::new(Some(account));
        json_string_result!(response)
    }

// FIXME: unused
//    pub(crate) fn auth_parse_query_account_resp_without_proof(
//        &self,
//        resp: &str,
//    ) -> IndyResult<String> {
//        self.parse_query_resp::<QueryAccountResponse>(resp)
//    }

    pub(crate) fn build_signed_message(
        &self,
        message: &[u8],
        did: &str,
        signature: &[u8],
    ) -> IndyResult<Vec<u8>> {
        let msg_write_request_payload = MsgWriteRequestPayload::from_proto_bytes(message)
            .to_indy(IndyErrorKind::InvalidState,
                     "Error while creating MsgWriteRequest object from string")?;

        MsgWriteRequest::from_payload(msg_write_request_payload)
            .add_signature(make_verification_id(did), &signature)
            .to_msg_bytes()
    }

    pub(crate) fn build_signed_txn(
        &self,
        doc: SignDoc,
        signature: Vec<u8>,
    ) -> IndyResult<Vec<u8>> {
        let tx: Raw = TxRaw {
            body_bytes: doc.body_bytes,
            auth_info_bytes: doc.auth_info_bytes,
            signatures: vec![signature],
        }.into();

        let tx_bytes: Vec<u8> = tx.to_bytes()?;
        Ok(tx_bytes)
    }
}


#[cfg(test)]
mod tests {
    use indy_api_types::errors::IndyErrorKind;
    use crate::services::CheqdLedgerService;
    use cosmrs::rpc::endpoint::abci_query;
    use failure::AsFail;

    #[async_std::test]
    async fn error_on_absent_account() {
        // Response with account_id which is placed in the ledger
        // let response_str = "{\"response\":{\"code\":0,\"log\":\"\",\"info\":\"\",\"index\":\"0\",\"key\":\"AU2mGX24tqcWKdw+ujCI5pTTjT0l\",\"value\":\"CiAvY29zbW9zLmF1dGgudjFiZXRhMS5CYXNlQWNjb3VudBIxCi1jb3Ntb3MxZmtucGpsZGNrNm4zdjJ3dTg2YXJwejh4am5mYzYwZjk5eWxjamQYAg==\",\"proof\":{\"ops\":[{\"field_type\":\"ics23:iavl\",\"key\":\"AU2mGX24tqcWKdw+ujCI5pTTjT0l\",\"data\":\"CoACChUBTaYZfbi2pxYp3D66MIjmlNONPSUSVQogL2Nvc21vcy5hdXRoLnYxYmV0YTEuQmFzZUFjY291bnQSMQotY29zbW9zMWZrbnBqbGRjazZuM3Yyd3U4NmFycHo4eGpuZmM2MGY5OXlsY2pkGAIaCwgBGAEgASoDAAICIikIARIlAgQCIJCiEpLGLTw3oUwhxhLthrSQgH6/ZWP6WCaD+4qaDiRRICIrCAESBAQIAiAaISB3lwHIMjW/jzRIbQtbBI894/yjTANfmdB8A/cY4CCMqSIrCAESBAgWAiAaISBplxd9W1qx9qgRrM7bBI1H8s4T2ZmHpmZRiXZPazKFsQ==\"},{\"field_type\":\"ics23:simple\",\"key\":\"YWNj\",\"data\":\"CtYBCgNhY2MSIOJakBCYIkbqTRCoAEDpSTnl7rGgNzzDLb0XscS55bKAGgkIARgBIAEqAQAiJwgBEgEBGiC2zBYtOhm67NjRq5Mao2OvPk9gAiNWUXnktEnJw48zhCInCAESAQEaILoD6gZnAzBWw9ZVknNCj3v/RqlcvuUEtfjTDMdO1ewlIicIARIBARogYHfOqhT4vz6WOZvqYQji+PZzn+iOMbO8URuv4ZMg6NUiJwgBEgEBGiAszcJa5DrW2vA27Uwywvi1WcHxukHGa8l13mgEA1Y8yw==\"}]},\"height\":\"5\",\"codespace\":\"\"}}";
        let response_str = "{\"response\":{\"code\":0,\"log\":\"\",\"info\":\"\",\"index\":\"0\",\"key\":\"AeGz6G1y6H1v1Kg8e2cGSgvz7NL4\",\"value\":\"\",\"proof\":{\"ops\":[{\"field_type\":\"ics23:iavl\",\"key\":\"AeGz6G1y6H1v1Kg8e2cGSgvz7NL4\",\"data\":\"EqAFChUB4bPobXLofW/UqDx7ZwZKC/Ps0vgSxwIKFQHfXrVYyt+ZUbcIz0mayQRjwlAdIRKdAQogL2Nvc21vcy5hdXRoLnYxYmV0YTEuQmFzZUFjY291bnQSeQotY29zbW9zMW1hMHQya3gybTd2NHJkY2dlYXllNGpneXYwcDlxOGZwZDhndnljEkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohApKTNiVduW3xZNSn+zQxxqnslZV/DhKWAXHpv7GfqPsGIAEaCwgBGAEgASoDAAICIisIARIEBAYCIBohIExK/7/GpCerk6tIH6neH5AxHxcYLqDzmeaC8f2tLCWrIikIARIlBg4CIAU6A3QFxXxGs2M3p4yFRb+v2C6iprxnccrKmpiJOKWRICIpCAESJQgWDiB1REtS2G7xNbVTz4I85WkugzFAJOmfArSt506sW7M2+iAavAIKFQHxgpZ221d2gulE/DST1FG2f/PinxJoCiIvY29zbW9zLmF1dGgudjFiZXRhMS5Nb2R1bGVBY2NvdW50EkIKMQotY29zbW9zMTd4cGZ2YWttMmFtZzk2MnlsczZmODR6M2tlbGw4YzVsc2VycXRhGAMSDWZlZV9jb2xsZWN0b3IaCwgBGAEgASoDAAICIisIARIEAgQCIBohIGZiNESsZqBLS4kNK4AqKKCga2VCRSIbT2/P6uarCh8wIikIARIlBAYCIOL0iWbzfbuUuHo6kY/RDEJzcBTvYiryMbh3+NhJvGuGICIpCAESJQYOAiAFOgN0BcV8RrNjN6eMhUW/r9guoqa8Z3HKypqYiTilkSAiKQgBEiUIFg4gdURLUthu8TW1U8+CPOVpLoMxQCTpnwK0redOrFuzNvog\"},{\"field_type\":\"ics23:simple\",\"key\":\"YWNj\",\"data\":\"CtYBCgNhY2MSIOBYamvCdoQNeiyk2JXOKH2Gp4xJ5NDxgtk2SUObRJWyGgkIARgBIAEqAQAiJwgBEgEBGiC/ElSJKxQ2ZQSGI8P4TAzAxkZYOjRw/0CWfqlsLBypRSInCAESAQEaIFh3Vqa4j1sJB+YGnconZJyneXHLvYRyA0SQdoF8mXsbIicIARIBARogXQMnB5H/NO2xg2mJEkcBbLxXKiXoTXef3Sp432W6O6siJwgBEgEBGiA9D80z3WD6/zCy5HDjilVuHtqnFSdhy/9JKVFUDYhBHg==\"}]},\"height\":\"863\",\"codespace\":\"\"}}";
        let _response: abci_query::Response = serde_json::from_str::<abci_query::Response>(response_str).unwrap();
        let cheqd_ledger_service = CheqdLedgerService::new();
        let err = cheqd_ledger_service.auth_parse_query_account_resp(&response_str).unwrap_err();
        assert!(err.to_string().contains(IndyErrorKind::QueryAccountDoesNotExist.as_fail().to_string().as_str()));
    }
}
