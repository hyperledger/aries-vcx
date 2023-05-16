use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::BaseLedger, wallet::base_wallet::BaseWallet,
};

use crate::core::profile::profile::Profile;

use super::{mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger, mock_wallet::MockWallet};

/// Implementation of a [Profile] which uses [MockLedger], [MockAnoncreds] and [MockWallet] to return
/// mock data for all Profile methods. Only for unit testing purposes
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

#[cfg(test)]
pub mod mocks {
    use std::sync::Arc;

    use aries_vcx_core::{
        anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::BaseLedger, wallet::base_wallet::BaseWallet,
    };

    use crate::core::profile::profile::Profile;

    #[derive(Default, Debug)]
    pub struct MockPartsProfile {
        ledger: Option<Arc<dyn BaseLedger>>,
        anoncreds: Option<Arc<dyn BaseAnonCreds>>,
        wallet: Option<Arc<dyn BaseWallet>>,
    }

    impl MockPartsProfile {
        pub fn set_ledger(self, ledger: Arc<dyn BaseLedger>) -> Self {
            MockPartsProfile {
                ledger: Some(ledger),
                ..self
            }
        }

        pub fn set_anoncreds(self, anoncreds: Arc<dyn BaseAnonCreds>) -> Self {
            MockPartsProfile {
                anoncreds: Some(anoncreds),
                ..self
            }
        }

        pub fn set_wallet(self, wallet: Arc<dyn BaseWallet>) -> Self {
            MockPartsProfile {
                wallet: Some(wallet),
                ..self
            }
        }
    }

    impl Profile for MockPartsProfile {
        fn inject_ledger(self: Arc<Self>) -> Arc<dyn BaseLedger> {
            self.ledger.as_ref().unwrap().clone()
        }

        fn inject_anoncreds(self: Arc<Self>) -> Arc<dyn BaseAnonCreds> {
            self.anoncreds.as_ref().unwrap().clone()
        }

        fn inject_wallet(&self) -> Arc<dyn BaseWallet> {
            self.wallet.as_ref().unwrap().clone()
        }
    }
}
