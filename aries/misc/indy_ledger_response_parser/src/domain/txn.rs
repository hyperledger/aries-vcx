use serde_json::Value;

use super::{
    constants::GET_TXN,
    response::{GetReplyResultV0, GetReplyResultV1, ReplyType},
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetTxnReplyResult {
    GetTxnReplyResultV0(GetReplyResultV0<Value>),
    GetTxnReplyResultV1(GetReplyResultV1<Value>),
}

impl ReplyType for GetTxnReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_TXN
    }
}
