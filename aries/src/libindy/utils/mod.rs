use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use indy_sys::CommandHandle;

use crate::settings;

pub mod ledger;
pub mod anoncreds;
pub mod signus;
pub mod wallet;
pub mod callback;
pub mod callback_u32;
pub mod pool;
pub mod crypto;
pub mod payments;
pub mod cache;
pub mod logger;

pub mod error_codes;

static COMMAND_HANDLE_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn next_command_handle() -> CommandHandle {
    (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as CommandHandle
}

lazy_static! {
    static ref LIBINDY_MOCK: Mutex<LibindyMock> = Mutex::new(LibindyMock::default());
}

#[derive(Default)]
pub struct LibindyMock {
    results: Vec<u32>
}

impl LibindyMock {
    pub fn set_next_result(rc: u32) {
        if settings::indy_mocks_enabled() {
            LIBINDY_MOCK.lock().unwrap().results.push(rc);
        }
    }

    pub fn get_result() -> u32 {
        LIBINDY_MOCK.lock().unwrap().results.pop().unwrap_or_default()
    }
}

#[allow(unused_imports)]
#[cfg(test)]
pub mod tests {
    use indy::future::Future;

    use crate::init::open_main_pool;
    use crate::settings;
    use crate::utils::devsetup::*;

    use super::*;

    // TODO:  Is used for Aries tests...try to remove and use one of devsetup's
    pub mod test_setup {
        use indy;

        use super::*;

        pub const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
        pub const WALLET_CREDENTIALS: &'static str = r#"{"key":"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY", "key_derivation_method":"RAW"}"#;

        pub struct WalletSetup {
            pub name: String,
            pub wallet_config: String,
            pub wh: indy::WalletHandle,
        }

        pub fn setup_wallet() -> WalletSetup {
            let name: String = crate::utils::random::generate_random_name();
            let wallet_config = json!({"id": name}).to_string();

            indy::wallet::create_wallet(&wallet_config, WALLET_CREDENTIALS).wait().unwrap();
            let wallet_handle = indy::wallet::open_wallet(&wallet_config, WALLET_CREDENTIALS).wait().unwrap();
            wallet::set_wallet_handle(wallet_handle);

            WalletSetup { name, wallet_config, wh: wallet_handle }
        }

        pub fn create_trustee_key(wallet_handle: indy::WalletHandle) -> String {
            let key_config = json!({"seed": TRUSTEE_SEED}).to_string();
            indy::crypto::create_key(wallet_handle, Some(&key_config)).wait().unwrap()
        }

        pub fn create_key(wallet_handle: indy::WalletHandle) -> String {
            let seed: String = crate::utils::random::generate_random_seed();
            let key_config = json!({"seed": seed}).to_string();
            indy::crypto::create_key(wallet_handle, Some(&key_config)).wait().unwrap()
        }

        impl Drop for WalletSetup {
            fn drop(&mut self) {
                if self.wh.0 != 0 {
                    indy::wallet::close_wallet(self.wh).wait().unwrap();
                    indy::wallet::delete_wallet(&self.wallet_config, WALLET_CREDENTIALS).wait().unwrap();
                }
            }
        }
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_pool_and_wallet() {
        let _setup_defaults = SetupDefaults::init();
        let setup_wallet = SetupWallet::init();
        let setup_pool = SetupPoolConfig::init();

        open_main_pool(&setup_pool.pool_config).unwrap();
        wallet::create_and_open_as_main_wallet(&setup_wallet.wallet_config).unwrap();
    }
}
