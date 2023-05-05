use std::sync::atomic::{AtomicUsize, Ordering};
use vdrtools::{types::validation::Validatable, CommandHandle};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

pub mod mocks;

static COMMAND_HANDLE_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn next_command_handle() -> CommandHandle {
    (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as CommandHandle
}

pub fn parse_and_validate<'a, T>(s: &'a str) -> VcxCoreResult<T>
where
    T: Validatable,
    T: serde::Deserialize<'a>,
{
    let data = serde_json::from_str::<T>(s)?;

    match data.validate() {
        Ok(_) => Ok(data),
        Err(s) => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::LibindyInvalidStructure,
            s,
        )),
    }
}

// TODO:  move to devsetup, see if we can reuse this / merge with different setup
pub mod test_setup {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    pub fn generate_random_name() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(25)
            .collect::<String>()
    }

    pub fn generate_random_seed() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect::<String>()
    }

    use crate::{indy, WalletHandle};

    const TRUSTEE_SEED: &str = "000000000000000000000000Trustee1";
    const WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";
    const WALLET_KEY_DERIVATION: &str = "RAW";

    pub async fn with_wallet<F>(f: impl FnOnce(WalletHandle) -> F)
    where
        F: std::future::Future<Output = ()>,
    {
        let wallet_config = indy::wallet::WalletConfig {
            wallet_name: generate_random_name(),
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
        let seed: String = generate_random_seed();

        indy::signing::create_key(wallet_handle, Some(&seed)).await.unwrap()
    }
}
