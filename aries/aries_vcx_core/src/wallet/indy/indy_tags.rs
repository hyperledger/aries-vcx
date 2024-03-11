use std::collections::HashMap;

use crate::wallet::record_tags::{RecordTag, RecordTags};

pub struct IndyTags(HashMap<String, String>);

impl IndyTags {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }

    pub fn from_record_tags(tags: RecordTags) -> Self {
        let mut map = HashMap::new();
        let tags_vec: Vec<_> = tags.into_iter().map(|tag| tag.into_pair()).collect();
        map.extend(tags_vec);
        Self(map)
    }

    pub fn into_record_tags(self) -> RecordTags {
        let mut items: Vec<_> = self
            .0
            .into_iter()
            .map(|(key, val)| RecordTag::new(&key, &val))
            .collect();
        items.sort();

        RecordTags::new(items)
    }
}
