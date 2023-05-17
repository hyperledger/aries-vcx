use std::collections::HashMap;

use serde::Deserialize;

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub req_id: u64,
    pub identifier: String,
    pub signature: Option<String>,
    pub signatures: Option<HashMap<String, String>>,
    pub endorser: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum Response {
    #[serde(rename = "REQNACK")]
    ReqNACK(Reject),
    #[serde(rename = "REJECT")]
    Reject(Reject),
    #[serde(rename = "REPLY")]
    Reply(Reply),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reject {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    ReplyV0(ReplyV0),
    ReplyV1(ReplyV1),
}

#[derive(Debug, Deserialize)]
pub struct ReplyV0 {
    pub result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1 {
    pub data: ReplyDataV1,
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1 {
    pub result: serde_json::Value,
}

pub fn verify_transaction_can_be_endorsed(transaction_json: &str, did: &str) -> VcxCoreResult<()> {
    let transaction: Request = serde_json::from_str(transaction_json)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, format!("{err:?}")))?;

    let transaction_endorser = transaction.endorser.ok_or(AriesVcxCoreError::from_msg(
        AriesVcxCoreErrorKind::InvalidJson,
        "Transaction cannot be endorsed: endorser DID is not set.",
    ))?;

    if transaction_endorser != did {
        return Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!(
                "Transaction cannot be endorsed: transaction endorser DID `{transaction_endorser}` and sender DID `{did}` are different"
            ),
        ));
    }

    let identifier = transaction.identifier.as_str();
    if transaction.signature.is_none()
        && !transaction
            .signatures
            .as_ref()
            .map(|signatures| signatures.contains_key(identifier))
            .unwrap_or(false)
    {
        return Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            "Transaction cannot be endorsed: the author must sign the transaction.".to_string(),
        ));
    }

    Ok(())
}
