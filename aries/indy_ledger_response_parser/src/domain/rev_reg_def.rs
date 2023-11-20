use indy_vdr::ledger::requests::rev_reg_def::RevocationRegistryDefinitionV1;

use super::{
    constants::GET_REVOC_REG_DEF,
    response::{GetReplyResultV1, ReplyType},
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetRevocRegDefReplyResult {
    GetRevocRegDefReplyResultV0(GetRevocRegDefResultV0),
    GetRevocRegDefReplyResultV1(GetReplyResultV1<RevocationRegistryDefinitionV1>),
}

impl ReplyType for GetRevocRegDefReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_REVOC_REG_DEF
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRevocRegDefResultV0 {
    pub seq_no: i32,
    pub data: RevocationRegistryDefinitionV1,
}
