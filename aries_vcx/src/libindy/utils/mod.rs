use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use vdrtools_sys::CommandHandle;

use crate::global::settings;

pub mod anoncreds;
pub mod cache;
pub mod crypto;
pub mod ledger;
pub mod logger;
pub mod mocks;
pub mod pool;
pub mod signus;
pub mod wallet;

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

// TODO:  move to devsetup, see if we can reuse this / merge with different setup
#[cfg(feature = "test_utils")]
pub mod test_setup {
    use vdrtools;

    pub const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
    pub const WALLET_CREDENTIALS: &'static str =
        r#"{"key":"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY", "key_derivation_method":"RAW"}"#;

    pub struct WalletSetup {
        pub name: String,
        pub wallet_config: String,
        pub wallet_handle: vdrtools::WalletHandle,
    }

    pub async fn setup_wallet() -> WalletSetup {
        let name: String = crate::utils::random::generate_random_name();
        let wallet_config = json!({ "id": name }).to_string();

        vdrtools::wallet::create_wallet(&wallet_config, WALLET_CREDENTIALS)
            .await
            .unwrap();
        let wallet_handle = vdrtools::wallet::open_wallet(&wallet_config, WALLET_CREDENTIALS)
            .await
            .unwrap();

        WalletSetup {
            name,
            wallet_config,
            wallet_handle,
        }
    }

    pub async fn create_trustee_key(wallet_handle: vdrtools::WalletHandle) -> String {
        let key_config = json!({ "seed": TRUSTEE_SEED }).to_string();
        vdrtools::crypto::create_key(wallet_handle, Some(&key_config))
            .await
            .unwrap()
    }

    pub async fn create_key(wallet_handle: vdrtools::WalletHandle) -> String {
        let seed: String = crate::utils::random::generate_random_seed();
        let key_config = json!({ "seed": seed }).to_string();
        vdrtools::crypto::create_key(wallet_handle, Some(&key_config))
            .await
            .unwrap()
    }

    impl Drop for WalletSetup {
        fn drop(&mut self) {
            if self.wallet_handle.0 != 0 {
                futures::executor::block_on(vdrtools::wallet::close_wallet(self.wallet_handle)).unwrap();
                futures::executor::block_on(vdrtools::wallet::delete_wallet(&self.wallet_config, WALLET_CREDENTIALS))
                    .unwrap();
            }
        }
    }
}
