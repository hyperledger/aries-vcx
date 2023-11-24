use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EntryTag {
    Encrypted(String, String),
    Plaintext(String, String),
}

#[cfg(feature = "vdrtools_wallet")]
impl From<EntryTag> for (String, String) {
    fn from(value: EntryTag) -> Self {
        match value {
            EntryTag::Encrypted(key, val) => (key, val),
            EntryTag::Plaintext(key, val) => (format!("~{}", key), val),
        }
    }
}

#[cfg(feature = "vdrtools_wallet")]
impl From<(String, String)> for EntryTag {
    fn from(value: (String, String)) -> Self {
        if value.0.starts_with('~') {
            EntryTag::Plaintext(value.0.trim_start_matches('~').into(), value.1)
        } else {
            EntryTag::Encrypted(value.0, value.1)
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EntryTags {
    inner: Vec<EntryTag>,
}

impl EntryTags {
    pub fn add(&mut self, tag: EntryTag) {
        self.inner.push(tag)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl IntoIterator for EntryTags {
    type Item = EntryTag;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<EntryTag> for EntryTags {
    fn from_iter<T: IntoIterator<Item = EntryTag>>(iter: T) -> Self {
        let mut tags = Self::default();

        for item in iter {
            tags.add(item);
        }
        tags
    }
}

impl From<Vec<EntryTag>> for EntryTags {
    fn from(value: Vec<EntryTag>) -> Self {
        value.into_iter().fold(Self::default(), |mut memo, item| {
            memo.add(item);
            memo
        })
    }
}

impl From<EntryTags> for Vec<EntryTag> {
    fn from(value: EntryTags) -> Self {
        value.inner
    }
}

#[cfg(feature = "vdrtools_wallet")]
impl From<EntryTags> for HashMap<String, String> {
    fn from(value: EntryTags) -> Self {
        let tags: Vec<EntryTag> = value.into();
        tags.into_iter().fold(Self::new(), |mut memo, item| {
            let (key, value) = item.into();

            memo.insert(key, value);
            memo
        })
    }
}

#[cfg(feature = "vdrtools_wallet")]
impl From<HashMap<String, String>> for EntryTags {
    fn from(value: HashMap<String, String>) -> Self {
        Self {
            inner: value
                .into_iter()
                .map(|(key, value)| (key, value))
                .map(From::from)
                .collect(),
        }
    }
}
