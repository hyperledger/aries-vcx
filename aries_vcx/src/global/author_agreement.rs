use crate::errors::error::VcxResult;
use aries_vcx_core::global::author_agreement::{
    get_txn_author_agreement, set_txn_author_agreement, TxnAuthorAgreementAcceptanceData,
};

pub fn proxy_set_txn_author_agreement(
    text: Option<String>,
    version: Option<String>,
    taa_digest: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u64,
) -> VcxResult<()> {
    set_txn_author_agreement(text, version, taa_digest, acc_mech_type, time_of_acceptance).map_err(|err| err.into())
}

pub fn proxy_get_txn_author_agreement() -> VcxResult<Option<TxnAuthorAgreementAcceptanceData>> {
    get_txn_author_agreement().map_err(|err| err.into())
}
