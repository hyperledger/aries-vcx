use std::collections::HashMap;

use crate::wallet::record_tags::{RecordTag, RecordTags};

pub(crate) struct IndyTags(HashMap<String, String>);

impl IndyTags {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }

    pub fn from_entry_tags(tags: RecordTags) -> Self {
        let mut map = HashMap::new();
        let tags_vec: Vec<_> = tags.into_iter().collect();
        map.extend(tags_vec);
        Self(map)
    }

    pub fn into_entry_tags(self) -> RecordTags {
        let mut items: Vec<RecordTag> = self.0.into_iter().collect();
        items.sort();

        RecordTags::new(items)
    }
}
