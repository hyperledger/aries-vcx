use std::sync::Arc;

use aries_vcx_core::ledger::base_ledger::{TaaConfigurator, TxnAuthrAgrmtOptions};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
    wallet::base_wallet::BaseWallet,
};

use super::{mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger, mock_wallet::MockWallet};
use crate::core::profile::profile::Profile;
use async_trait::async_trait;

/// Implementation of a [Profile] which uses [MockLedger], [MockAnoncreds] and [MockWallet] to return
/// mock data for all Profile methods. Only for unit testing purposes
#[derive(Debug)]
pub struct MockProfile;

#[async_trait]
impl Profile for MockProfile {
    fn inject_indy_ledger_write(self: Arc<Self>) -> Arc<dyn IndyLedgerWrite> {
        Arc::new(MockLedger {})
    }

    fn inject_indy_ledger_read(self: Arc<Self>) -> Arc<dyn IndyLedgerRead> {
        Arc::new(MockLedger {})
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        Arc::new(MockAnoncreds {})
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::new(MockWallet {})
    }

    fn inject_anoncreds_ledger_read(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerRead> {
        Arc::new(MockLedger {})
    }

    fn inject_anoncreds_ledger_write(self: Arc<Self>) -> Arc<dyn AnoncredsLedgerWrite> {
        Arc::new(MockLedger {})
    }

    async fn update_taa_configuration(self: Arc<Self>, taa_options: TxnAuthrAgrmtOptions) {
        error!("update_taa_configuration not implemented for MockProfile")
    }
}
