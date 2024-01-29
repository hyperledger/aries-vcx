use typed_builder::TypedBuilder;

use crate::wallet::{base_wallet::record_category::RecordCategory, entry_tag::EntryTags};

#[derive(Debug, Default, Clone, TypedBuilder)]
pub struct Record {
    category: RecordCategory,
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

    pub fn category(&self) -> &RecordCategory {
        &self.category
    }

    pub fn tags(&self) -> &EntryTags {
        &self.tags
    }
}
