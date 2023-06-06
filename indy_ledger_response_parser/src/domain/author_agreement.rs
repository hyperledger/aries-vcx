use indy_vdr::ledger::requests::author_agreement::AcceptanceMechanisms;

use super::{
    constants::GET_TXN_AUTHR_AGRMT,
    response::{GetReplyResultV0, ReplyType},
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum GetTxnAuthorAgreementResult {
    GetTxnAuthorAgreementResultV1(GetReplyResultV0<GetTxnAuthorAgreementResultV1>),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GetTxnAuthorAgreementResultV1 {
    pub text: String,
    pub version: String,
    pub aml: Option<AcceptanceMechanisms>,
    pub digest: Option<String>,
    pub ratification_ts: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTxnAuthorAgreementData {
    pub text: String,
    pub version: String,
    pub aml: Option<AcceptanceMechanisms>,
    pub digest: Option<String>,
    pub ratification_ts: Option<u64>,
}

impl ReplyType for GetTxnAuthorAgreementResult {
    fn get_type<'a>() -> &'a str {
        GET_TXN_AUTHR_AGRMT
    }
}
