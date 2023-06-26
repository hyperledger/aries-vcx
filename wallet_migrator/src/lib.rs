pub mod credx2anoncreds;
pub mod error;
pub mod vdrtools2credx;

use std::fmt::Display;

use error::MigrationResult;
use log::{debug, error, info, warn};
pub use vdrtools::{
    types::domain::wallet::{Config, Credentials, KeyDerivationMethod, Record},
    Locator, WalletHandle,
};

/// Retrieves all records from a wallet and migrates them
/// by applying the `migrate_fn` argument.
///
/// The migrated records are inserted into a newly created
/// wallet, based on the provided `config` and `credentials`.
pub async fn migrate_wallet<E>(
    wallet_handle: WalletHandle,
    config: Config,
    credentials: Credentials,
    migrate_fn: impl FnMut(Record) -> Result<Option<Record>, E>,
) -> MigrationResult<()>
where
    E: Display,
{
    info!("Starting wallet migration...");
    let locator = Locator::instance();

    debug!("Creating new wallet {} ...", config.id);
    locator
        .wallet_controller
        .create(config.clone(), credentials.clone())
        .await?;

    debug!("Opening new wallet...");

    let new_wh = locator
        .wallet_controller
        .open(config.clone(), credentials.clone())
        .await?;

    debug!("Migrating records from wallet with handle {wallet_handle:?} to wallet with handle {new_wh:?}");

    let res = locator
        .wallet_controller
        .migrate_records(wallet_handle, new_wh, migrate_fn)
        .await;

    locator.wallet_controller.close(new_wh).await?;

    if let Err(e) = &res {
        error!("Migration error encountered: {e}");

        match locator.wallet_controller.delete(config, credentials).await.ok() {
            Some(_) => debug!("Newly created wallet deleted"),
            None => warn!("Could not delete newly created wallet!"),
        };
    }

    res?;

    Ok(())
}
