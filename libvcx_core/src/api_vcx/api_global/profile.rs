use std::sync::Arc;

use aries_vcx::{
    self,
    aries_vcx_core::{
        anoncreds::base_anoncreds::BaseAnonCreds,
        ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite},
        wallet::base_wallet::BaseWallet,
    },
};
use aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, TaaConfigurator, TxnAuthrAgrmtOptions,
};

use crate::{
    api_vcx::api_global::{
        pool::{GLOBAL_LEDGER_INDY_READ, GLOBAL_LEDGER_INDY_WRITE},
        wallet::{GLOBAL_BASE_ANONCREDS, GLOBAL_BASE_WALLET},
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

pub fn try_get_main_wallet() -> LibvcxResult<Option<Arc<impl BaseWallet>>> {
    let base_wallet = GLOBAL_BASE_WALLET.read()?;
    base_wallet.as_ref().cloned().map(Some).ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
    })
}

pub fn get_main_wallet() -> LibvcxResult<Arc<impl BaseWallet>> {
    let base_wallet = GLOBAL_BASE_WALLET.read()?;
    base_wallet.as_ref().cloned().ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
    })
}

pub fn get_main_anoncreds() -> LibvcxResult<Arc<impl BaseAnonCreds>> {
    let anoncreds = GLOBAL_BASE_ANONCREDS.read()?;
    anoncreds.as_ref().cloned().ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Anoncreds is not initialized")
    })
}

pub fn get_main_ledger_read() -> LibvcxResult<Arc<impl IndyLedgerRead + AnoncredsLedgerRead>> {
    let ledger = GLOBAL_LEDGER_INDY_READ.read()?;
    ledger.as_ref().cloned().ok_or_else(|| {
        LibvcxError::from_msg(
            LibvcxErrorKind::NotReady,
            "Anoncreds ledger read is not initialized",
        )
    })
}

pub fn get_main_ledger_write() -> LibvcxResult<Arc<impl IndyLedgerWrite + AnoncredsLedgerWrite>> {
    let ledger = GLOBAL_LEDGER_INDY_WRITE.read()?;
    ledger.as_ref().cloned().ok_or_else(|| {
        LibvcxError::from_msg(
            LibvcxErrorKind::NotReady,
            "Indy ledger write is not initialized",
        )
    })
}

pub fn update_taa_configuration(taa_options: TxnAuthrAgrmtOptions) -> LibvcxResult<()> {
    let configurator = GLOBAL_LEDGER_INDY_WRITE.read()?;
    match configurator.as_ref() {
        None => Err(LibvcxError::from_msg(
            LibvcxErrorKind::NotReady,
            "Ledger is not initialized",
        ))?,
        Some(configurator) => configurator.set_txn_author_agreement_options(taa_options)?,
    };
    Ok(())
}

pub fn get_taa_configuration() -> LibvcxResult<Option<TxnAuthrAgrmtOptions>> {
    let configurator = GLOBAL_LEDGER_INDY_WRITE.read()?;
    match configurator.as_ref() {
        None => Err(LibvcxError::from_msg(
            LibvcxErrorKind::NotReady,
            "Ledger is not initialized",
        ))?,
        Some(configurator) => configurator
            .get_txn_author_agreement_options()
            .map_err(|err| err.into()),
    }
}
