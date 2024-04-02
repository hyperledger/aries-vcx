use std::str::FromStr;

use aries_askar::entry::{Entry, EntryKind};

use crate::{
    errors::error::VcxWalletError,
    wallet::base_wallet::{record::Record, record_category::RecordCategory},
};

impl TryFrom<Entry> for Record {
    type Error = VcxWalletError;

    fn try_from(entry: Entry) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .category(RecordCategory::from_str(&entry.category)?)
            .name(entry.name)
            .value(String::from_utf8(entry.value.to_vec())?)
            .tags(entry.tags.into())
            .build())
    }
}

impl From<Record> for Entry {
    fn from(record: Record) -> Self {
        Self {
            category: record.category().to_string(),
            name: record.name().to_string(),
            value: record.value().into(),
            kind: EntryKind::Item,
            tags: record.tags().clone().into_iter().map(From::from).collect(),
        }
    }
}
