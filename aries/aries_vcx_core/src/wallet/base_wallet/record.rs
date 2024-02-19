use typed_builder::TypedBuilder;

use super::record_category::RecordCategory;
use crate::wallet::record_tags::RecordTags;

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
