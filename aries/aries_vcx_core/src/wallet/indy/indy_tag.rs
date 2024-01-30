use std::collections::HashMap;

use crate::wallet::entry_tag::{EntryTag, EntryTags};
struct IndyTag((String, String));

impl IndyTag {
    pub fn new(pair: (String, String)) -> Self {
        Self(pair)
    }

    pub fn into_inner(self) -> (String, String) {
        self.0
    }

    pub fn into_entry_tag(self) -> EntryTag {
        let inner = self.into_inner();

        EntryTag::from_pair(inner)
    }

    pub fn from_entry_tag(tag: EntryTag) -> Self {
        match tag {
            EntryTag::Tag(key, val) => Self((key, val)),
        }
    }
}

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
        let tags_vec: Vec<_> = tags
            .into_iter()
            .map(|tag| IndyTag::from_entry_tag(tag).into_inner())
            .collect();
        map.extend(tags_vec);
        Self(map)
    }

    pub fn into_entry_tags(self) -> EntryTags {
        let mut items: Vec<EntryTag> = self
            .0
            .into_iter()
            .map(|pair| IndyTag::new(pair).into_entry_tag())
            .collect();
        items.sort();

        EntryTags::new(items)
    }
}
