pub mod ledger;
#[cfg(all(feature = "credx", feature = "vdrtools_wallet"))]
pub mod modular_libs_profile;
#[cfg(feature = "vdr_proxy_ledger")]
pub mod vdr_proxy_profile;

use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::{
        base_ledger::{
            AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
            TxnAuthrAgrmtOptions,
        },
        indy_vdr_ledger::GetTxnAuthorAgreementData,
    },
    wallet::base_wallet::BaseWallet,
};
use async_trait::async_trait;

use crate::errors::error::VcxResult;

const DEFAULT_AML_LABEL: &str = "eula";

pub async fn prepare_taa_options(
    ledger_read: Arc<dyn IndyLedgerRead>,
) -> VcxResult<Option<TxnAuthrAgrmtOptions>> {
    if let Some(taa_result) = ledger_read.get_txn_author_agreement().await? {
        let taa_result: GetTxnAuthorAgreementData = serde_json::from_str(&taa_result)?;
        Ok(Some(TxnAuthrAgrmtOptions {
            version: taa_result.version,
            text: taa_result.text,
            mechanism: DEFAULT_AML_LABEL.to_string(),
        }))
    } else {
        Ok(None)
    }
}

#[async_trait]
pub trait Profile: std::fmt::Debug + Send + Sync {
    type LedgerRead: IndyLedgerRead + AnoncredsLedgerRead;
    type LedgerWrite: IndyLedgerWrite + AnoncredsLedgerWrite;
    type Anoncreds: BaseAnonCreds;
    type Wallet: BaseWallet;

    fn ledger_read(&self) -> &Self::LedgerRead;

    fn ledger_write(&self) -> &Self::LedgerWrite;

    fn anoncreds(&self) -> &Self::Anoncreds;

    fn wallet(&self) -> &Self::Wallet;

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()>;
}
