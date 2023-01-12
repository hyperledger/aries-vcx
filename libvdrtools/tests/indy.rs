#![cfg_attr(feature = "fatal_warnings", deny(warnings))]

use vdrtoolsrs as vdrtools;

#[test]
fn set_runtime_config_works() {
    vdrtools::set_runtime_config(r#"{"crypto_thread_pool_size": 2}"#);
}
