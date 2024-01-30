use async_trait::async_trait;
use typed_builder::TypedBuilder;

use crate::{errors::error::VcxCoreResult, wallet::entry_tag::EntryTags};

#[cfg(feature = "vdrtools_wallet")]
use vdrtools::WalletRecord;

#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct Record {
    category: String,
    name: String,
    value: String,
    #[builder(default)]
    tags: EntryTags,
}

impl Record {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn tags(&self) -> &EntryTags {
        &self.tags
    }
}

#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct PartialRecord {
    category: Option<String>,
    name: String,
    value: Option<String>,
    #[builder(default)]
    tags: Option<EntryTags>,
}

impl PartialRecord {
    pub fn value(&self) -> &Option<String> {
        &self.value
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &Option<String> {
        &self.category
    }

    pub fn tags(&self) -> &Option<EntryTags> {
        &self.tags
    }

    #[cfg(feature = "vdrtools_wallet")]
    pub fn from_wallet_record(wallet_record: WalletRecord) -> Self {
        use crate::wallet::indy::indy_tag::IndyTags;

        let name = wallet_record.get_id().into();
        let category = wallet_record.get_type();
        let value = wallet_record.get_value();

        let found_tags = wallet_record.get_tags();

        Self::builder()
            .name(name)
            .category(category.map(Into::into))
            .value(value.map(Into::into))
            .tags(found_tags.map(|tags| IndyTags::new(tags.clone()).into_entry_tags()))
            .build()
    }
}

#[async_trait]
pub trait AllRecords {
    fn total_count(&self) -> VcxCoreResult<Option<usize>>;
    async fn next(&mut self) -> VcxCoreResult<Option<PartialRecord>>;
}
