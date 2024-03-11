use async_trait::async_trait;
use typed_builder::TypedBuilder;

use super::record_category::RecordCategory;
use crate::{errors::error::VcxCoreResult, wallet::record_tags::RecordTags};

#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct Record {
    category: RecordCategory,
    name: String,
    value: String,
    #[builder(default)]
    tags: RecordTags,
}

impl Record {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category(&self) -> &RecordCategory {
        &self.category
    }

    pub fn tags(&self) -> &RecordTags {
        &self.tags
    }
}

#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct PartialRecord {
    category: Option<String>,
    name: String,
    value: Option<String>,
    #[builder(default)]
    tags: Option<RecordTags>,
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

    pub fn tags(&self) -> &Option<RecordTags> {
        &self.tags
    }
}

#[async_trait]
pub trait AllRecords {
    fn total_count(&self) -> VcxCoreResult<Option<usize>>;
    async fn next(&mut self) -> VcxCoreResult<Option<PartialRecord>>;
}
