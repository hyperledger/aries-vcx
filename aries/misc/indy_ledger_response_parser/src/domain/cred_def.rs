use indy_vdr::{
    ledger::{
        identifiers::{CredentialDefinitionId, SchemaId},
        requests::cred_def::{CredentialDefinitionData, SignatureType},
    },
    utils::did::ShortDidValue,
};

use super::{
    constants::GET_CRED_DEF,
    response::{GetReplyResultV1, ReplyType},
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetCredDefReplyResult {
    GetCredDefReplyResultV0(GetCredDefResultV0),
    GetCredDefReplyResultV1(GetReplyResultV1<GetCredDefResultDataV1>),
}

impl ReplyType for GetCredDefReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_CRED_DEF
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetCredDefResultV0 {
    pub identifier: ShortDidValue,
    #[serde(rename = "ref")]
    pub ref_: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    pub signature_type: SignatureType,
    pub origin: ShortDidValue,
    pub tag: Option<String>,
    pub data: CredentialDefinitionData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCredDefResultDataV1 {
    #[allow(unused)] // unused, but part of entity
    pub ver: String,
    pub id: CredentialDefinitionId,
    #[serde(rename = "type")]
    pub type_: SignatureType,
    pub tag: String,
    pub schema_ref: SchemaId,
    pub public_keys: CredentialDefinitionData,
}
