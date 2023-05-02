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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod unit_tests {
    use crate::utils::devsetup::SetupDefaults;

    use super::*;

    const TEXT: &str = "indy agreement";
    const VERSION: &str = "1.0.0";
    const ACCEPTANCE_MECHANISM: &str = "acceptance mechanism label 1";
    const TIME_OF_ACCEPTANCE: u64 = 123456789;

    #[test]
    fn set_txn_author_agreement_works() {
        let _setup = SetupDefaults::init();

        assert!(settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT).is_err());

        set_txn_author_agreement(
            Some(TEXT.to_string()),
            Some(VERSION.to_string()),
            None,
            ACCEPTANCE_MECHANISM.to_string(),
            TIME_OF_ACCEPTANCE,
        )
        .unwrap();

        assert!(settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT).is_ok());
    }

    #[test]
    fn get_txn_author_agreement_works() {
        let _setup = SetupDefaults::init();

        set_txn_author_agreement(
            Some(TEXT.to_string()),
            Some(VERSION.to_string()),
            None,
            ACCEPTANCE_MECHANISM.to_string(),
            TIME_OF_ACCEPTANCE,
        )
        .unwrap();

        let meta = get_txn_author_agreement().unwrap().unwrap();

        let expected_meta = TxnAuthorAgreementAcceptanceData {
            text: Some(TEXT.to_string()),
            version: Some(VERSION.to_string()),
            taa_digest: None,
            acceptance_mechanism_type: ACCEPTANCE_MECHANISM.to_string(),
            time_of_acceptance: TIME_OF_ACCEPTANCE,
        };

        assert_eq!(expected_meta, meta);
    }

    #[test]
    fn get_txn_author_agreement_works_for_not_set() {
        let _setup = SetupDefaults::init();

        assert!(get_txn_author_agreement().unwrap().is_none());
    }
}
