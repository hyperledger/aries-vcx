use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

use vdrtools::CommandHandle;

use crate::global::settings;

pub mod mocks;

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

// todo: get rid of this, we no longer deal with rc return codes from vdrtools
//      (this is leftover from times when we talked to vdrtool via FFI)
impl LibindyMock {
    pub fn set_next_result(rc: u32) {
        if settings::indy_mocks_enabled() {
            LIBINDY_MOCK
                .lock()
                .expect("Unabled to access LIBINDY_MOCK")
                .results
                .push(rc);
        }
    }

    pub fn get_result() -> u32 {
        LIBINDY_MOCK
            .lock()
            .expect("Unable to access LIBINDY_MOCK")
            .results
            .pop()
            .unwrap_or_default()
    }
}

// TODO:  move to devsetup, see if we can reuse this / merge with different setup
#[cfg(feature = "test_utils")]
pub mod test_setup {

    use crate::indy;

    const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
    const WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
    const WALLET_KEY_DERIVATION: &str = "RAW";

    pub async fn with_wallet<F>(f: impl FnOnce(vdrtools::WalletHandle) -> F)
    where
        F: std::future::Future<Output = ()>,
    {
        let wallet_config = indy::wallet::WalletConfig {
            wallet_name: crate::utils::random::generate_random_name(),
            wallet_key: WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KEY_DERIVATION.into(),
            ..Default::default()
        };

        indy::wallet::create_indy_wallet(&wallet_config).await.unwrap();

        let wallet_handle = indy::wallet::open_wallet(&wallet_config).await.unwrap();

        f(wallet_handle).await;

        indy::wallet::close_wallet(wallet_handle).await.unwrap();

        indy::wallet::delete_wallet(&wallet_config).await.unwrap();
    }

    pub async fn create_trustee_key(wallet_handle: vdrtools::WalletHandle) -> String {
        indy::signing::create_key(wallet_handle, Some(TRUSTEE_SEED))
            .await
            .unwrap()
    }

    pub async fn create_key(wallet_handle: vdrtools::WalletHandle) -> String {
        let seed: String = crate::utils::random::generate_random_seed();

        indy::signing::create_key(wallet_handle, Some(&seed)).await.unwrap()
    }
}
