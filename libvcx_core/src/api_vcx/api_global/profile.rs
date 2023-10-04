use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use aries_vcx::{
    aries_vcx_core::{
        anoncreds::base_anoncreds::BaseAnonCreds,
        ledger::base_ledger::{
            AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
            TxnAuthrAgrmtOptions,
        },
        wallet::{base_wallet::BaseWallet, mock_wallet::MockWallet},
    },
    utils::mockdata::profile::{
        mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger, mock_profile::MockProfile,
    },
};

use crate::{
    api_vcx::api_global::{
        pool::{
            GLOBAL_LEDGER_ANONCREDS_READ, GLOBAL_LEDGER_ANONCREDS_WRITE, GLOBAL_LEDGER_INDY_READ,
            GLOBAL_LEDGER_INDY_WRITE, GLOBAL_TAA_CONFIGURATOR,
        },
        wallet::{GLOBAL_BASE_ANONCREDS, GLOBAL_BASE_WALLET},
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

pub trait ProfileV2: Send + Sync {
    fn inject_indy_ledger_read(&self) -> LibvcxResult<Arc<dyn IndyLedgerRead>>;

    fn inject_indy_ledger_write(&self) -> LibvcxResult<Arc<dyn IndyLedgerWrite>>;

    fn inject_anoncreds(&self) -> LibvcxResult<Arc<dyn BaseAnonCreds>>;

    fn inject_anoncreds_ledger_read(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerRead>>;

    fn inject_anoncreds_ledger_write(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerWrite>>;

    fn inject_wallet(&self) -> LibvcxResult<Arc<dyn BaseWallet>>;

    fn try_inject_wallet(&self) -> LibvcxResult<Option<Arc<dyn BaseWallet>>>;

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> LibvcxResult<()>;

    fn get_taa_configuration(&self) -> LibvcxResult<Option<TxnAuthrAgrmtOptions>>;
}

#[derive(Clone)]
struct VcxGlobalsProfile {}

impl Debug for VcxGlobalsProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VcxGlobalsProfile")
    }
}

impl ProfileV2 for VcxGlobalsProfile {
    fn inject_indy_ledger_read(&self) -> LibvcxResult<Arc<dyn IndyLedgerRead>> {
        let ledger = GLOBAL_LEDGER_INDY_READ.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Indy ledger read is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_indy_ledger_write(&self) -> LibvcxResult<Arc<dyn IndyLedgerWrite>> {
        let ledger = GLOBAL_LEDGER_INDY_WRITE.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Indy ledger write is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_anoncreds(&self) -> LibvcxResult<Arc<dyn BaseAnonCreds>> {
        let anoncreds = GLOBAL_BASE_ANONCREDS.read()?;
        match anoncreds.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Anoncreds is not initialized",
            )),
            Some(a) => Ok(Arc::clone(a)),
        }
    }

    fn inject_anoncreds_ledger_read(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerRead>> {
        let ledger = GLOBAL_LEDGER_ANONCREDS_READ.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Anoncreds ledger read is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_anoncreds_ledger_write(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerWrite>> {
        let ledger = GLOBAL_LEDGER_ANONCREDS_WRITE.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Anoncreds ledger write is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_wallet(&self) -> LibvcxResult<Arc<dyn BaseWallet>> {
        let base_wallet = GLOBAL_BASE_WALLET.read()?;
        match base_wallet.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Wallet is not initialized",
            )),
            Some(w) => Ok(Arc::clone(w)),
        }
    }

    fn try_inject_wallet(&self) -> LibvcxResult<Option<Arc<dyn BaseWallet>>> {
        let base_wallet = GLOBAL_BASE_WALLET.read()?;
        base_wallet
            .as_ref()
            .map(|w| Some(Arc::clone(w)))
            .ok_or_else(|| {
                LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized")
            })
    }

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> LibvcxResult<()> {
        let configurator = GLOBAL_TAA_CONFIGURATOR.read()?;
        match configurator.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Ledger is not initialized",
            ))?,
            Some(configurator) => configurator
                .as_ref()
                .set_txn_author_agreement_options(taa_options)?,
        };
        Ok(())
    }

    fn get_taa_configuration(&self) -> LibvcxResult<Option<TxnAuthrAgrmtOptions>> {
        let configurator = GLOBAL_TAA_CONFIGURATOR.read()?;
        match configurator.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Ledger is not initialized",
            ))?,
            Some(configurator) => configurator
                .as_ref()
                .get_txn_author_agreement_options()
                .map_err(|err| err.into()),
        }
    }
}

lazy_static! {
    static ref GLOBAL_PROFILE: VcxGlobalsProfile = VcxGlobalsProfile {};
}

impl ProfileV2 for MockProfile {
    fn inject_indy_ledger_read(&self) -> LibvcxResult<Arc<dyn IndyLedgerRead>> {
        Ok(Arc::new(MockLedger {}))
    }

    fn inject_indy_ledger_write(&self) -> LibvcxResult<Arc<dyn IndyLedgerWrite>> {
        Ok(Arc::new(MockLedger {}))
    }

    fn inject_anoncreds(&self) -> LibvcxResult<Arc<dyn BaseAnonCreds>> {
        Ok(Arc::new(MockAnoncreds {}))
    }

    fn inject_anoncreds_ledger_read(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerRead>> {
        Ok(Arc::new(MockLedger {}))
    }

    fn inject_anoncreds_ledger_write(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerWrite>> {
        Ok(Arc::new(MockLedger {}))
    }

    fn inject_wallet(&self) -> LibvcxResult<Arc<dyn BaseWallet>> {
        Ok(Arc::new(MockWallet {}))
    }

    fn try_inject_wallet(&self) -> LibvcxResult<Option<Arc<dyn BaseWallet>>> {
        Ok(Some(Arc::new(MockWallet {})))
    }

    fn update_taa_configuration(&self, _taa_options: TxnAuthrAgrmtOptions) -> LibvcxResult<()> {
        Ok(())
    }

    fn get_taa_configuration(&self) -> LibvcxResult<Option<TxnAuthrAgrmtOptions>> {
        Ok(Some(TxnAuthrAgrmtOptions {
            text: "foo".to_string(),
            version: "bar".to_string(),
            mechanism: "baz".to_string(),
        }))
    }
}

pub fn get_main_profile() -> Arc<dyn ProfileV2> {
    Arc::new(GLOBAL_PROFILE.clone())
}

pub fn try_get_main_wallet() -> LibvcxResult<Option<Arc<dyn BaseWallet>>> {
    get_main_profile().try_inject_wallet()
}

pub fn get_main_wallet() -> LibvcxResult<Arc<dyn BaseWallet>> {
    get_main_profile().inject_wallet()
}

pub fn get_main_anoncreds() -> LibvcxResult<Arc<dyn BaseAnonCreds>> {
    get_main_profile().inject_anoncreds()
}

pub fn get_main_indy_ledger_read() -> LibvcxResult<Arc<dyn IndyLedgerRead>> {
    get_main_profile().inject_indy_ledger_read()
}

pub fn get_main_indy_ledger_write() -> LibvcxResult<Arc<dyn IndyLedgerWrite>> {
    get_main_profile().inject_indy_ledger_write()
}

pub fn get_main_anoncreds_ledger_read() -> LibvcxResult<Arc<dyn AnoncredsLedgerRead>> {
    get_main_profile().inject_anoncreds_ledger_read()
}

pub fn get_main_anoncreds_ledger_write() -> LibvcxResult<Arc<dyn AnoncredsLedgerWrite>> {
    get_main_profile().inject_anoncreds_ledger_write()
}
