use std::sync::Arc;

use aries_vcx::{
    self,
    aries_vcx_core::{
        anoncreds::base_anoncreds::BaseAnonCreds,
        ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite},
    },
};
#[cfg(feature = "askar_wallet")]
use aries_vcx_core::wallet::askar::AskarWallet;
#[cfg(feature = "vdrtools_wallet")]
use aries_vcx_core::wallet::indy::IndySdkWallet;
use aries_vcx_core::{
    ledger::base_ledger::{
        AnoncredsLedgerRead, AnoncredsLedgerWrite, TaaConfigurator, TxnAuthrAgrmtOptions,
    },
    wallet::base_wallet::BaseWallet,
};

#[cfg(feature = "askar_wallet")]
use super::wallet::askar::GLOBAL_ASKAR_WALLET;
#[cfg(feature = "vdrtools_wallet")]
use super::wallet::indy::GLOBAL_INDY_WALLET;
use crate::{
    api_vcx::api_global::{
        pool::{GLOBAL_LEDGER_INDY_READ, GLOBAL_LEDGER_INDY_WRITE},
        wallet::GLOBAL_BASE_ANONCREDS,
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

#[cfg(feature = "vdrtools_wallet")]
pub fn try_get_main_indy_wallet() -> LibvcxResult<Option<Arc<IndySdkWallet>>> {
    let base_wallet = GLOBAL_INDY_WALLET.read()?;
    base_wallet.as_ref().cloned().map(Some).ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
    })
}

#[cfg(feature = "askar_wallet")]
pub fn try_get_main_askar_wallet() -> LibvcxResult<Option<Arc<AskarWallet>>> {
    let base_wallet = GLOBAL_ASKAR_WALLET.read()?;
    base_wallet.as_ref().cloned().map(Some).ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
    })
}

#[allow(unused_variables)]
pub fn try_get_main_wallet() -> LibvcxResult<Option<Arc<impl BaseWallet>>> {
    #[cfg(feature = "vdrtools_wallet")]
    let wallet = try_get_main_indy_wallet()?;

    #[cfg(feature = "askar_wallet")]
    let wallet = try_get_main_askar_wallet()?;

    Ok(wallet)
}

#[cfg(feature = "askar_wallet")]
pub fn get_main_askar_wallet() -> LibvcxResult<Arc<impl BaseWallet>> {
    let base_wallet = GLOBAL_ASKAR_WALLET.read()?;
    base_wallet.as_ref().cloned().ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
    })
}

#[cfg(feature = "vdrtools_wallet")]
pub fn get_main_indy_wallet() -> LibvcxResult<Arc<impl BaseWallet>> {
    let base_wallet = GLOBAL_INDY_WALLET.read()?;
    base_wallet.as_ref().cloned().ok_or_else(|| {
        LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
    })
}

#[allow(unused_variables)]
pub fn get_main_wallet() -> LibvcxResult<Arc<impl BaseWallet>> {
    #[cfg(feature = "vdrtools_wallet")]
    let wallet = get_main_indy_wallet()?;

    #[cfg(feature = "askar_wallet")]
    let wallet = get_main_askar_wallet()?;

    Ok(wallet)
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
