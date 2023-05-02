use serde_json;
use aries_vcx_core::utils::author_agreement::{get_global_txn_author_agreement, set_global_txn_author_agreement, TxnAuthorAgreementAcceptanceData};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::global::settings;

pub fn set_txn_author_agreement(
    text: Option<String>,
    version: Option<String>,
    taa_digest: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u64,
) -> VcxResult<()> {
    set_global_txn_author_agreement(text, version, taa_digest, acc_mech_type, time_of_acceptance)
        .map_err(|err| err.into())
}

pub fn get_txn_author_agreement() -> VcxResult<Option<TxnAuthorAgreementAcceptanceData>> {
    get_global_txn_author_agreement()
        .map_err(|err| err.into())
}
