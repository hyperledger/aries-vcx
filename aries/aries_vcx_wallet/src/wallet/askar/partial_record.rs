use super::askar_utils::value_from_entry;
use crate::{
    errors::error::VcxWalletResult,
    wallet::{
        askar::askar_utils::{local_key_to_bs58_private_key, local_key_to_bs58_public_key},
        base_wallet::{
            key_value::KeyValue, record::PartialRecord, record_category::RecordCategory,
        },
    },
};

impl PartialRecord {
    pub fn from_askar_entry(entry: aries_askar::entry::Entry) -> VcxWalletResult<Self> {
        Ok(Self::builder()
            .name(entry.name.clone())
            .category(Some(entry.category.clone()))
            .value(Some(value_from_entry(entry.clone())?))
            .tags(Some(entry.tags.into()))
            .build())
    }

    pub fn from_askar_key_entry(key_entry: aries_askar::kms::KeyEntry) -> VcxWalletResult<Self> {
        let local_key = key_entry.load_local_key()?;
        let name = key_entry.name();
        let tags = key_entry.tags_as_slice();

        let value = KeyValue::new(
            local_key_to_bs58_private_key(&local_key)?,
            local_key_to_bs58_public_key(&local_key)?,
        );

        let value = serde_json::to_string(&value)?;

        Ok(Self::builder()
            .name(name.into())
            .category(Some(RecordCategory::Key.to_string()))
            .value(Some(value))
            .tags(Some(tags.to_vec().into()))
            .build())
    }
}
