use env_logger;
use std::sync::Once;

use crate::agency_settings;
use crate::mocking;
use crate::mocking::AgencyMockDecrypted;
use crate::utils::wallet::reset_wallet_handle;

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
    agency_settings::clear_config_agency();
    agency_settings::set_testing_defaults_agency();
}

impl SetupMocks {
    pub fn init() -> SetupMocks {
        setup();
        mocking::enable_agency_mocks();
        ::std::env::set_var("DUMMY_TEST_MODE", "true");
        SetupMocks
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        AgencyMockDecrypted::clear_mocks();
        ::std::env::set_var("DUMMY_TEST_MODE", "false");
        reset_wallet_handle();
        mocking::disable_agency_mocks();
    }
}

impl SetupWallet {
    pub fn init() -> SetupWallet {
        SetupWallet
    }
}

impl Drop for SetupWallet {
    fn drop(&mut self) {
    }
}

