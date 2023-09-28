use std::env;

use crate::domain::IndyConfig;

pub struct ConfigController {}

impl ConfigController {
    pub(crate) fn new() -> ConfigController {
        ConfigController {}
    }

    /// Set libvdrtools runtime configuration. Can be optionally called to change current params.
    ///
    /// #Params
    /// config: {
    ///     "crypto_thread_pool_size": Optional<int> - size of thread pool for the most expensive
    /// crypto operations. (4 by default)     "collect_backtrace": Optional<bool> - whether
    /// errors backtrace should be collected.         Capturing of backtrace can affect library
    /// performance.         NOTE: must be set before invocation of any other API functions.
    /// }
    ///
    /// #Errors
    /// Common*
    pub fn set_runtime_config(&self, config: IndyConfig) {
        trace!("set_runtime_config > {:?}", config);

        // FIXME: Deprecate this param.
        if let Some(_crypto_thread_pool_size) = config.crypto_thread_pool_size {
            warn!("indy_set_runtime_config ! unsupported param used");
        }

        match config.collect_backtrace {
            Some(true) => env::set_var("RUST_BACKTRACE", "1"),
            Some(false) => env::set_var("RUST_BACKTRACE", "0"),
            _ => {}
        }

        trace!("set_runtime_config <");
    }
}
