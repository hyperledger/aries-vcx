use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use indy_sys::CommandHandle;

use crate::global::settings;

pub mod ledger;
pub mod anoncreds;
pub mod signus;
pub mod wallet;
pub mod pool;
pub mod crypto;
pub mod cache;
pub mod logger;
pub mod mocks;

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
    results: Vec<u32>,
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

// TODO:  Is used for Aries tests...try to remove and use one of devsetup's
#[cfg(feature = "test_utils")]
pub mod test_setup {
    use indy;
    use crate::global;

    use super::*;

    pub const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
    pub const WALLET_CREDENTIALS: &'static str = r#"{"key":"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY", "key_derivation_method":"RAW"}"#;

    pub struct WalletSetup {
        pub name: String,
        pub wallet_config: String,
        pub wallet_handle: indy::WalletHandle,
    }

    pub async fn setup_wallet() -> WalletSetup {
        let name: String = crate::utils::random::generate_random_name();
        let wallet_config = json!({"id": name}).to_string();

        indy::wallet::create_wallet(&wallet_config, WALLET_CREDENTIALS).await.unwrap();
        let wallet_handle = indy::wallet::open_wallet(&wallet_config, WALLET_CREDENTIALS).await.unwrap();
        global::wallet::set_wallet_handle(wallet_handle);

        WalletSetup { name, wallet_config, wallet_handle: wallet_handle }
    }

    pub async fn create_trustee_key(wallet_handle: indy::WalletHandle) -> String {
        let key_config = json!({"seed": TRUSTEE_SEED}).to_string();
        indy::crypto::create_key(wallet_handle, Some(&key_config)).await.unwrap()
    }

    pub async fn create_key(wallet_handle: indy::WalletHandle) -> String {
        let seed: String = crate::utils::random::generate_random_seed();
        let key_config = json!({"seed": seed}).to_string();
        indy::crypto::create_key(wallet_handle, Some(&key_config)).await.unwrap()
    }

    impl Drop for WalletSetup {
        fn drop(&mut self) {
            if self.wallet_handle.0 != 0 {
                futures::executor::block_on(indy::wallet::close_wallet(self.wallet_handle)).unwrap();
                futures::executor::block_on(indy::wallet::delete_wallet(&self.wallet_config, WALLET_CREDENTIALS)).unwrap();
            }
        }
    }
}

#[allow(unused_imports)]
#[cfg(feature = "pool_tests")]
pub mod tests {
    use crate::global;
    use crate::global::pool::open_main_pool;
    use crate::global::settings;
    use crate::utils::devsetup::*;

    use super::*;

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_init_pool_and_wallet() {
        let _setup_defaults = SetupDefaults::init();
        let setup_wallet = SetupWallet::init().await;
        let setup_pool = SetupPoolConfig::init().await;

        open_main_pool(&setup_pool.pool_config).await.unwrap();
        global::wallet::create_and_open_as_main_wallet(&setup_wallet.wallet_config).await.unwrap();
    }
}
