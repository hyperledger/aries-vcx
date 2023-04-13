use std::sync::atomic::{AtomicUsize, Ordering};
use vdrtools::CommandHandle;

pub mod mocks;

static COMMAND_HANDLE_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn next_command_handle() -> CommandHandle {
    (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as CommandHandle
}

// TODO:  move to devsetup, see if we can reuse this / merge with different setup
#[cfg(feature = "test_utils")]
pub mod test_setup {

    use crate::{indy, WalletHandle};

    const TRUSTEE_SEED: &'static str = "000000000000000000000000Trustee1";
    const WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
    const WALLET_KEY_DERIVATION: &str = "RAW";

    pub async fn with_wallet<F>(f: impl FnOnce(WalletHandle) -> F)
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

    pub async fn create_trustee_key(wallet_handle: WalletHandle) -> String {
        indy::signing::create_key(wallet_handle, Some(TRUSTEE_SEED))
            .await
            .unwrap()
    }

    pub async fn create_key(wallet_handle: WalletHandle) -> String {
        let seed: String = crate::utils::random::generate_random_seed();

        indy::signing::create_key(wallet_handle, Some(&seed)).await.unwrap()
    }
}
