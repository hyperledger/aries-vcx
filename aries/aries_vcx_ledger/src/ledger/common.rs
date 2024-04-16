use std::collections::HashMap;

use did_parser_nom::Did;
use serde::Deserialize;

use crate::errors::error::{VcxLedgerError, VcxLedgerResult};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Request {
    #[allow(dead_code)]
    pub req_id: u64,
    pub identifier: String,
    pub signature: Option<String>,
    pub signatures: Option<HashMap<String, String>>,
    pub endorser: Option<String>,
}

pub fn verify_transaction_can_be_endorsed(
    transaction_json: &str,
    submitter_did: &Did,
) -> VcxLedgerResult<()> {
    let transaction: Request = serde_json::from_str(transaction_json)?;

    let endorser_did = transaction.endorser.ok_or(VcxLedgerError::InvalidState(
        "Transaction cannot be endorsed: endorser DID is not set.".into(),
    ))?;

    if &Did::parse(endorser_did.clone())? != submitter_did {
        return Err(VcxLedgerError::InvalidState(format!(
            "Transaction cannot be endorsed: transaction endorser DID `{endorser_did}` and sender \
             DID `{submitter_did}` are different"
        )));
    }

    let identifier = transaction.identifier.as_str();
    if transaction.signature.is_none()
        && !transaction
            .signatures
            .as_ref()
            .map(|signatures| signatures.contains_key(identifier))
            .unwrap_or(false)
    {
        return Err(VcxLedgerError::InvalidState(
            "Transaction cannot be endorsed: the author must sign the transaction.".to_string(),
        ));
    }

    Ok(())
}
