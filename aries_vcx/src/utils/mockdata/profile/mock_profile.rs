use std::sync::Arc;

use super::{mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger, mock_wallet::MockWallet};
use crate::{
    core::profile::profile::Profile,
    plugins::{
        anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::BaseLedger, wallet::base_wallet::BaseWallet,
    },
};

/// Implementation of a [Profile] which uses [MockLedger], [MockAnoncreds] and [MockWallet] to
/// return mock data for all Profile methods. Only for unit testing purposes
#[derive(Debug)]
pub struct MockProfile;

impl Profile for MockProfile {
    fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger> {
        Arc::new(MockLedger {})
    }

    fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
        Arc::new(MockAnoncreds {})
    }

    fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
        Arc::new(MockWallet {})
    }
}
