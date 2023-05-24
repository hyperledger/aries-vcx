use indy_vdr::ledger::requests::author_agreement::AcceptanceMechanisms;

use super::{constants::GET_TXN_AUTHR_AGRMT, response::ReplyType};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxnAuthorAgreementResult {
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
