use std::sync::Once;

use crate::testing::{mocking, mocking::AgencyMockDecrypted};

pub struct SetupMocks;

pub struct SetupWallet;

lazy_static! {
    static ref TEST_LOGGING_INIT: Once = Once::new();
}

pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        env_logger::init();
    })
}

fn setup() {
    init_test_logging();
}

impl SetupMocks {
    pub fn init() -> SetupMocks {
        setup();
        mocking::enable_agency_mocks();
        SetupMocks
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        AgencyMockDecrypted::clear_mocks();
        mocking::disable_agency_mocks();
    }
}

impl SetupWallet {
    pub fn init() -> SetupWallet {
        SetupWallet
    }
}

impl Drop for SetupWallet {
    fn drop(&mut self) {}
}
