#[cfg(feature = "migration")]
use aries_vcx_core::WalletHandle;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
        TxnAuthrAgrmtOptions,
    },
    wallet::base_wallet::BaseWallet,
};
use async_trait::async_trait;

use crate::errors::error::VcxResult;

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

    #[cfg(feature = "migration")]
    fn wallet_handle(&self) -> Option<WalletHandle> {
        None
    }

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> VcxResult<()>;
}
