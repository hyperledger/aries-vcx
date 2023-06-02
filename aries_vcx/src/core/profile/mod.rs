#[cfg(feature = "mixed_breed")]
pub mod mixed_breed_profile;
#[cfg(feature = "modular_libs")]
pub mod modular_libs_profile;
pub mod profile;
#[cfg(feature = "vdr_proxy_ledger")]
pub mod vdr_proxy_profile;
#[cfg(feature = "vdrtools")]
pub mod vdrtools_profile;

const DEFAULT_AML_LABEL: &str = "eula";

use std::sync::Arc;

#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
use aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
use aries_vcx_core::ledger::{base_ledger::IndyLedgerRead, indy_vdr_ledger::GetTxnAuthorAgreementData};

use crate::errors::error::VcxResult;

#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
pub async fn prepare_taa_options(ledger_read: Arc<dyn IndyLedgerRead>) -> VcxResult<Option<TxnAuthrAgrmtOptions>> {
    if let Some(taa_result) = ledger_read.get_txn_author_agreement().await? {
        let taa_result: GetTxnAuthorAgreementData = serde_json::from_str(&taa_result)?;
        Ok(Some(TxnAuthrAgrmtOptions {
            version: taa_result.version,
            text: taa_result.text,
            aml_label: DEFAULT_AML_LABEL.to_string(),
        }))
    } else {
        Ok(None)
    }
}
