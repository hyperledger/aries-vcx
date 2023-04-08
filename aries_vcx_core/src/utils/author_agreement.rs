use serde::{Deserialize, Serialize};
use serde_json;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
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
) -> VcxResult<()> {
    let meta = TxnAuthorAgreementAcceptanceData {
        text,
        version,
        taa_digest,
        acceptance_mechanism_type: acc_mech_type,
        time_of_acceptance,
    };

    let meta =
        serde_json::to_string(&meta).map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidOption, err))?;

    settings::set_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT, &meta)?;

    Ok(())
}

pub fn get_txn_author_agreement() -> VcxResult<Option<TxnAuthorAgreementAcceptanceData>> {
    trace!("get_txn_author_agreement >>>");
    match settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT) {
        Ok(value) => {
            let meta: TxnAuthorAgreementAcceptanceData = serde_json::from_str(&value)
                .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err))?;
            Ok(Some(meta))
        }
        Err(_) => Ok(None),
    }
}
