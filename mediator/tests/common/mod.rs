#![allow(dead_code)]

pub mod test_setup {
    // inspired by
    // https://stackoverflow.com/questions/58006033/how-to-run-setup-code-before-any-tests-run-in-rust
    use std::sync::Once;

    static INIT: Once = Once::new();
    pub trait OneTimeInit {
        // runs the initialization code if it hasn't been run yet, else does nothing
        fn init(&self) {
            INIT.call_once(|| self.one_time_setup_code());
        }
        // your custom initialization code goes here
        fn one_time_setup_code(&self) -> ();
    }
}

pub mod prelude {
    pub use anyhow::Result;
    pub use log::{debug, error, info};
    pub use url::Url;
}
