use std::str::FromStr;

use log::{error, info, trace, warn};

use super::{
    record::{PartialRecord, Record},
    BaseWallet,
};
use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{base_wallet::record_category::RecordCategory, record_tags::RecordTags},
};

#[derive(Debug, Clone, Copy)]
pub struct MigrationStats {
    pub migrated: u32,
    pub skipped: u32,
    pub duplicated: u32,
    pub failed: u32,
}

pub async fn migrate_records<E>(
    src_wallet: &impl BaseWallet,
    dest_wallet: &impl BaseWallet,
    mut migrate_fn: impl FnMut(Record) -> Result<Option<Record>, E>,
) -> VcxWalletResult<MigrationStats>
where
    E: std::fmt::Display,
{
    let mut records = src_wallet.all_records().await?;
    let total = records.total_count()?;
    info!("Migrating {total:?} records");
    let mut num_record = 0;
    let mut migration_stats = MigrationStats {
        migrated: 0,
        skipped: 0,
        duplicated: 0,
        failed: 0,
    };

    while let Some(source_record) = records.next().await? {
        num_record += 1;
        if num_record % 1000 == 1 {
            warn!(
                "Migrating wallet record number {num_record} / {total:?}, intermediary migration \
                 result: ${migration_stats:?}"
            );
        }

        trace!("Migrating record: {:?}", source_record);
        let maybe_record =
            transform_record(num_record, source_record.clone(), &mut migration_stats);

        if let Some(some_record) = maybe_record {
            let migrated_record = match migrate_fn(some_record) {
                Ok(record) => match record {
                    None => {
                        warn!("Skipping non-migratable record ({num_record}): {source_record:?}");
                        migration_stats.skipped += 1;
                        continue;
                    }
                    Some(record) => record,
                },
                Err(err) => {
                    warn!(
                        "Skipping item due failed item migration, record ({num_record}): \
                         {source_record:?}, err: {err}"
                    );
                    migration_stats.failed += 1;
                    continue;
                }
            };

            if migrated_record.is_key() {
                add_key(dest_wallet, &mut migration_stats, migrated_record).await
            } else {
                add_record(dest_wallet, &mut migration_stats, migrated_record).await
            }
        }
    }

    warn!("Migration of total {total:?} records completed, result: ${migration_stats:?}");
    Ok(migration_stats)
}

fn transform_record(
    num_record: i32,
    source_record: PartialRecord,
    migration_stats: &mut MigrationStats,
) -> Option<Record> {
    let category = match &source_record.category() {
        None => {
            warn!("Skipping item missing 'type' field, record ({num_record}): {source_record:?}");
            migration_stats.skipped += 1;
            return None;
        }
        Some(cat) => match RecordCategory::from_str(cat) {
            Ok(record_category) => record_category,
            Err(_) => {
                warn!(
                    "Skipping item due to invalid category, record ({num_record}): \
                     {source_record:?}"
                );
                migration_stats.skipped += 1;
                return None;
            }
        },
    };
    let value = match &source_record.value() {
        None => {
            warn!("Skipping item missing 'value' field, record ({num_record}): {source_record:?}");
            migration_stats.skipped += 1;
            return None;
        }
        Some(value) => value.clone(),
    };
    let tags = match source_record.tags() {
        None => RecordTags::default(),
        Some(tags) => tags.clone(),
    };

    let record = Record::builder()
        .category(category)
        .name(source_record.name().into())
        .value(value)
        .tags(tags)
        .build();

    info!("Migrating wallet record {record:?}");

    Some(record)
}

async fn add_key(
    new_wallet: &impl BaseWallet,
    migration_stats: &mut MigrationStats,
    key_record: Record,
) {
    let key_value = match key_record.key_value() {
        Ok(val) => val,
        Err(err) => {
            error!("Error parsing key value for {key_record:?}, is this record a key?: {err:?}");
            migration_stats.failed += 1;
            return;
        }
    };

    match new_wallet
        .create_key(key_record.name(), key_value, key_record.tags())
        .await
    {
        Err(err) => {
            error!("Error adding key {key_record:?} to destination wallet: {err:?}");
            migration_stats.failed += 1;
        }
        Ok(_) => {
            migration_stats.migrated += 1;
        }
    }
}

async fn add_record(
    new_wallet: &impl BaseWallet,
    migration_stats: &mut MigrationStats,
    record: Record,
) {
    match new_wallet.add_record(record.clone()).await {
        Err(err) => match err {
            VcxWalletError::DuplicateRecord(_) => {
                trace!("Record type: {record:?} already exists in destination wallet, skipping");
                migration_stats.duplicated += 1;
            }
            _ => {
                error!("Error adding record {record:?} to destination wallet: {err:?}");
                migration_stats.failed += 1;
            }
        },
        Ok(()) => {
            migration_stats.migrated += 1;
        }
    }
}
