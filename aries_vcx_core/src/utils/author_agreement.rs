use serde::{Deserialize, Serialize};
use serde_json;

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use crate::global::settings;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TxnAuthorAgreementAcceptanceData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taa_digest: Option<String>,
    pub acceptance_mechanism_type: String,
    pub time_of_acceptance: u64,
}

pub fn set_txn_author_agreement(
    text: Option<String>,
    version: Option<String>,
    taa_digest: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u64,
) -> VcxCoreResult<()> {
    let meta = TxnAuthorAgreementAcceptanceData {
        text,
        version,
        taa_digest,
        acceptance_mechanism_type: acc_mech_type,
        time_of_acceptance,
    };

    let meta = serde_json::to_string(&meta)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidOption, err))?;

    settings::set_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT, &meta)?;

    Ok(())
}

pub fn get_txn_author_agreement() -> VcxCoreResult<Option<TxnAuthorAgreementAcceptanceData>> {
    trace!("get_txn_author_agreement >>>");
    match settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT) {
        Ok(value) => {
            let meta: TxnAuthorAgreementAcceptanceData = serde_json::from_str(&value)
                .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err))?;
            Ok(Some(meta))
        }
        Err(_) => Ok(None),
    }
}
