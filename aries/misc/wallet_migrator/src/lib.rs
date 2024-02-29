pub mod error;
pub mod migrate2askar;

use aries_vcx_core::wallet::base_wallet::{
    migrate::{migrate_records, MigrationStats},
    record::Record,
    BaseWallet,
};
use error::{MigrationError, MigrationResult};
use log::info;
use migrate2askar::migrate_any_record_to_askar;
pub use vdrtools::types::domain::wallet::IndyRecord;

pub enum MigrationKind {
    ToAskar,
    Unknown,
}

impl MigrationKind {
    pub fn select_migrate_fn(
        &self,
    ) -> MigrationResult<impl FnMut(Record) -> Result<Option<Record>, MigrationError>> {
        match self {
            MigrationKind::ToAskar => Ok(migrate_any_record_to_askar),
            MigrationKind::Unknown => Err(error::MigrationError::Unsupported),
        }
    }
}

/// Retrieves all records from the source wallet and migrates them
/// by applying the `migrate_fn` argument. The records are then
/// placed in the destination wallet.
pub async fn migrate_wallet(
    src_wallet: &impl BaseWallet,
    dest_wallet: &impl BaseWallet,
    migration_kind: MigrationKind,
) -> MigrationResult<MigrationStats> {
    info!("Starting wallet migration");

    info!(
        "Migrating records from wallet with handle {src_wallet:?} to wallet with handle \
         {dest_wallet:?}"
    );
    let migrate_fn = migration_kind.select_migrate_fn()?;
    let res = migrate_records(src_wallet, dest_wallet, migrate_fn).await?;

    info!("Migration completed");

    Ok(res)
}
