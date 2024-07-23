use indy_vdr::utils::did::ShortDidValue;

use super::response::GetReplyResultV1;

#[allow(unused)]
// unused for now, but domain defined: https://github.com/hyperledger/indy-node/blob/main/docs/source/transactions.md#attrib
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetAttrReplyResult {
    GetAttrReplyResultV0(GetAttResultV0),
    GetAttrReplyResultV1(GetReplyResultV1<GetAttResultDataV1>),
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAttResultV0 {
    pub identifier: ShortDidValue,
    pub data: String,
    pub dest: ShortDidValue,
    pub raw: String,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetAttResultDataV1 {
    pub ver: String,
    pub id: String,
    pub did: ShortDidValue,
    pub raw: String,
}
