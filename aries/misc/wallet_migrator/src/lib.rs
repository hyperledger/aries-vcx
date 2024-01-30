pub mod credx2anoncreds;
pub mod error;
pub mod vdrtools2credx;

use std::fmt::Display;

use aries_vcx_core::wallet::base_wallet::{migrate::migrate_records, record::Record, BaseWallet};
use error::MigrationResult;
use log::info;
pub use vdrtools::types::domain::wallet::IndyRecord;

/// Retrieves all records from the source wallet and migrates them
/// by applying the `migrate_fn` argument. The records are then
/// placed in the destination wallet.
pub async fn migrate_wallet<E>(
    src_wallet: &impl BaseWallet,
    dest_wallet: &impl BaseWallet,
    migrate_fn: impl FnMut(Record) -> Result<Option<Record>, E>,
) -> MigrationResult<()>
where
    E: Display,
{
    info!("Starting wallet migration");

    info!(
        "Migrating records from wallet with handle {src_wallet:?} to wallet with handle \
         {dest_wallet:?}"
    );

    migrate_records(src_wallet, dest_wallet, migrate_fn).await?;

    info!("Migration completed");

    Ok(())
}
