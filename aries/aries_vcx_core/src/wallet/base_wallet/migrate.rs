use super::{record::Record, BaseWallet};
use crate::{
    errors::error::{AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::record_tags::RecordTags,
};

#[derive(Debug)]
pub struct MigrationStats {
    migrated: u32,
    skipped: u32,
    duplicated: u32,
    failed: u32,
}

pub async fn migrate_records<E>(
    src_wallet: &impl BaseWallet,
    dest_wallet: &impl BaseWallet,
    mut migrate_fn: impl FnMut(Record) -> Result<Option<Record>, E>,
) -> VcxCoreResult<MigrationStats>
where
    E: std::fmt::Display,
{
    let mut records = src_wallet.all_records().await?;
    let total = records.total_count();
    info!("Migrating {total:?} records");
    let mut num_record = 0;
    let mut migration_result = MigrationStats {
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
                 result: ${migration_result:?}"
            );
        }
        trace!("Migrating record: {:?}", source_record);
        let unwrapped_category = match &source_record.category() {
            None => {
                warn!(
                    "Skipping item missing 'category' field, record ({num_record}): \
                     {source_record:?}"
                );
                migration_result.skipped += 1;
                continue;
            }
            Some(type_) => type_.clone(),
        };
        let unwrapped_value = match &source_record.value() {
            None => {
                warn!(
                    "Skipping item missing 'value' field, record ({num_record}): {source_record:?}"
                );
                migration_result.skipped += 1;
                continue;
            }
            Some(value) => value.clone(),
        };
        let unwrapped_tags = match &source_record.tags() {
            None => RecordTags::default(),
            Some(tags) => tags.clone(),
        };

        let record = Record::builder()
            .category(unwrapped_category.parse()?)
            .name(source_record.name().to_string())
            .value(unwrapped_value)
            .tags(unwrapped_tags)
            .build();

        let migrated_record = match migrate_fn(record) {
            Ok(record) => match record {
                None => {
                    warn!("Skipping non-migratable record ({num_record}): {source_record:?}");
                    migration_result.skipped += 1;
                    continue;
                }
                Some(record) => record,
            },
            Err(err) => {
                warn!(
                    "Skipping item due failed item migration, record ({num_record}): \
                     {source_record:?}, err: {err}"
                );
                migration_result.failed += 1;
                continue;
            }
        };

        match dest_wallet.add_record(migrated_record.clone()).await {
            Err(err) => match err.kind() {
                AriesVcxCoreErrorKind::DuplicationWalletRecord => {
                    trace!(
                        "Record type: {migrated_record:?} already exists in destination wallet, \
                         skipping"
                    );
                    migration_result.duplicated += 1;
                    continue;
                }
                _ => {
                    error!(
                        "Error adding record {migrated_record:?} to destination wallet: {err:?}"
                    );
                    migration_result.failed += 1;
                    return Err(err);
                }
            },
            Ok(()) => {
                migration_result.migrated += 1;
            }
        }
    }
    warn!("Migration of total {total:?} records completed, result: ${migration_result:?}");
    Ok(migration_result)
}
