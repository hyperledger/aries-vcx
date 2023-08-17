use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLockReadGuard};

use crate::api_vcx::api_global::pool::{
    global_ledger_anoncreds_read, global_ledger_anoncreds_write, global_ledger_indy_read, global_ledger_indy_write,
    global_taa_configurator,
};
use crate::api_vcx::api_global::wallet::{global_base_anoncreds, global_base_wallet};
use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use aries_vcx::aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx::aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite, TaaConfigurator, TxnAuthrAgrmtOptions,
};
use aries_vcx::aries_vcx_core::wallet::indy::IndySdkWallet;
use aries_vcx::aries_vcx_core::wallet::mock_wallet::MockWallet;
use aries_vcx::aries_vcx_core::{wallet::base_wallet::BaseWallet, WalletHandle};
use aries_vcx::core::profile::{profile::Profile, vdrtools_profile::VdrtoolsProfile};
use aries_vcx::errors::error::VcxResult;
use aries_vcx::utils::mockdata::profile::mock_anoncreds::MockAnoncreds;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx::{global::settings::indy_mocks_enabled, utils::mockdata::profile::mock_profile::MockProfile};

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
        let ledger = global_ledger_indy_read.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Indy ledger read is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_indy_ledger_write(&self) -> LibvcxResult<Arc<dyn IndyLedgerWrite>> {
        let ledger = global_ledger_indy_write.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Indy ledger write is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_anoncreds(&self) -> LibvcxResult<Arc<dyn BaseAnonCreds>> {
        let anoncreds = global_base_anoncreds.read()?;
        match anoncreds.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Anoncreds is not initialized",
            )),
            Some(a) => Ok(Arc::clone(a)),
        }
    }

    fn inject_anoncreds_ledger_read(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerRead>> {
        let ledger = global_ledger_anoncreds_read.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Anoncreds ledger read is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_anoncreds_ledger_write(&self) -> LibvcxResult<Arc<dyn AnoncredsLedgerWrite>> {
        let ledger = global_ledger_anoncreds_write.read()?;
        match ledger.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Anoncreds ledger write is not initialized",
            )),
            Some(l) => Ok(Arc::clone(l)),
        }
    }

    fn inject_wallet(&self) -> LibvcxResult<Arc<dyn BaseWallet>> {
        let base_wallet = global_base_wallet.read()?;
        match base_wallet.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Wallet is not initialized",
            )),
            Some(w) => Ok(Arc::clone(w)),
        }
    }

    fn try_inject_wallet(&self) -> LibvcxResult<Option<Arc<dyn BaseWallet>>> {
        let base_wallet = global_base_wallet.read()?;
        base_wallet
            .as_ref()
            .map(|w| Some(Arc::clone(w)))
            .ok_or_else(|| LibvcxError::from_msg(LibvcxErrorKind::NotReady, "Wallet is not initialized"))
    }

    fn update_taa_configuration(&self, taa_options: TxnAuthrAgrmtOptions) -> LibvcxResult<()> {
        let configurator = global_taa_configurator.read()?;
        match configurator.as_ref() {
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::NotReady,
                "Ledger is not initialized",
            ))?,
            Some(configurator) => configurator.as_ref().set_txn_author_agreement_options(taa_options)?,
        };
        Ok(())
    }

    fn get_taa_configuration(&self) -> LibvcxResult<Option<TxnAuthrAgrmtOptions>> {
        let configurator = global_taa_configurator.read()?;
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
    static ref global_profile: VcxGlobalsProfile = VcxGlobalsProfile {};
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
    if indy_mocks_enabled() {
        return Arc::new(MockProfile {});
    }
    Arc::new(global_profile.clone())
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
