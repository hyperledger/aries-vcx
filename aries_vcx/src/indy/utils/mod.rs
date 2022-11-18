use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use vdrtools::CommandHandle;

use crate::global::settings;

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

// TODO:  move to devsetup, see if we can reuse this / merge with different setup
#[cfg(feature = "test_utils")]
pub mod test_setup {

    use crate::indy;

    const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
    const WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
    const WALLET_KEY_DERIVATION: &str = "RAW";

    pub struct WalletSetup {
        pub wallet_config: indy::wallet::WalletConfig,
        pub wallet_handle: vdrtools::WalletHandle,
    }

    pub async fn setup_wallet() -> WalletSetup {
        let wallet_config = indy::wallet::WalletConfig{
            wallet_name: crate::utils::random::generate_random_name(),
            wallet_key: WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KEY_DERIVATION.into(),
            .. Default::default()
        };

        indy::wallet::create_indy_wallet(&wallet_config)
            .await
            .unwrap();

        let wallet_handle = indy::wallet::open_wallet(&wallet_config)
            .await
            .unwrap();

        WalletSetup {
            wallet_config,
            wallet_handle,
        }
    }

    pub async fn create_trustee_key(wallet_handle: vdrtools::WalletHandle) -> String {
        indy::signing::create_key(wallet_handle, Some(TRUSTEE_SEED))
            .await
            .unwrap()
    }

    pub async fn create_key(wallet_handle: vdrtools::WalletHandle) -> String {
        let seed: String = crate::utils::random::generate_random_seed();

        indy::signing::create_key(wallet_handle, Some(&seed))
            .await
            .unwrap()
    }

    impl Drop for WalletSetup {
        fn drop(&mut self) {
            if self.wallet_handle.0 != 0 {
                tokio::runtime::Handle::current().block_on(async {
                    indy::wallet::close_wallet(self.wallet_handle)
                        .await
                        .unwrap();

                    indy::wallet::delete_wallet(&self.wallet_config)
                        .await
                        .unwrap();
                })
            }
        }
    }
}
