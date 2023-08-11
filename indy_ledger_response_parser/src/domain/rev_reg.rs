use super::constants::{GET_REVOC_REG, GET_REVOC_REG_DELTA};

use anoncreds_clsignatures::RevocationRegistry;
use indy_vdr::ledger::{identifiers::RevocationRegistryId, requests::rev_reg::RevocationRegistryV1};

use super::response::{GetReplyResultV1, ReplyType};

use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetRevocRegReplyResult {
    GetRevocRegReplyResultV0(GetRevocRegResultV0),
    GetRevocRegReplyResultV1(GetReplyResultV1<GetRevocRegDataV1>),
}

impl ReplyType for GetRevocRegReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_REVOC_REG
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRevocRegResultV0 {
    pub seq_no: i32,
    pub revoc_reg_def_id: RevocationRegistryId,
    pub data: RevocationRegistryV1,
    pub txn_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRevocRegDataV1 {
    pub revoc_reg_def_id: RevocationRegistryId,
    pub value: RevocationRegistryV1,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDeltaData {
    pub value: RevocationRegistryDeltaValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevocationRegistryDeltaValue {
    pub accum_from: Option<AccumulatorState>,
    pub accum_to: AccumulatorState,
    pub issued: HashSet<u32>,
    pub revoked: HashSet<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccumulatorState {
    pub value: RevocationRegistry,
    pub txn_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetRevocRegDeltaReplyResult {
    GetRevocRegDeltaReplyResultV0(GetRevocRegDeltaResultV0),
    GetRevocRegDeltaReplyResultV1(GetReplyResultV1<GetRevocRegDeltaDataV1>),
}

impl ReplyType for GetRevocRegDeltaReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_REVOC_REG_DELTA
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRevocRegDeltaResultV0 {
    pub seq_no: i32,
    pub revoc_reg_def_id: RevocationRegistryId,
    pub data: RevocationRegistryDeltaData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRevocRegDeltaDataV1 {
    pub revoc_reg_def_id: RevocationRegistryId,
    pub value: RevocationRegistryDeltaData,
}
