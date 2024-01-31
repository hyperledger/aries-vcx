use std::collections::HashMap;

use crate::wallet::entry_tags::{EntryTag, EntryTags};

pub(crate) struct IndyTags(HashMap<String, String>);

impl IndyTags {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }

    pub fn from_entry_tags(tags: EntryTags) -> Self {
        let mut map = HashMap::new();
        let tags_vec: Vec<_> = tags.into_iter().collect();
        map.extend(tags_vec);
        Self(map)
    }

    pub fn into_entry_tags(self) -> EntryTags {
        let mut items: Vec<EntryTag> = self.0.into_iter().collect();
        items.sort();

        EntryTags::new(items)
    }
}
