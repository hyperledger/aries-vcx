use std::fmt;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum EntryTag {
    Tag(String, String),
}

impl EntryTag {
    pub fn from_pair(pair: (String, String)) -> Self {
        Self::Tag(pair.0, pair.1)
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Tag(key, _) => key,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EntryTags {
    inner: Vec<EntryTag>,
}

impl Serialize for EntryTags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for tag in self.inner.iter() {
            match tag {
                EntryTag::Tag(key, val) => map.serialize_entry(&key, &val)?,
            }
        }
        map.end()
    }
}

struct EntryTagsVisitor;

impl<'de> Visitor<'de> for EntryTagsVisitor {
    type Value = EntryTags;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map representing tags")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut tags = EntryTags::new(vec![]);

        while let Some(pair) = map.next_entry()? {
            tags.add(EntryTag::from_pair(pair));
        }

        Ok(tags)
    }
}

impl<'de> Deserialize<'de> for EntryTags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(EntryTagsVisitor)
    }
}

impl EntryTags {
    pub fn new(inner: Vec<EntryTag>) -> Self {
        let mut items = inner;
        items.sort();

        Self { inner: items }
    }

    pub fn add(&mut self, tag: EntryTag) {
        self.inner.push(tag);
        self.inner.sort();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn into_inner(self) -> Vec<EntryTag> {
        self.inner
    }

    pub fn merge(&mut self, other: EntryTags) {
        self.inner.extend(other.into_inner());
        self.inner.sort();
    }

    pub fn remove(&mut self, tag: EntryTag) {
        self.inner
            .retain(|existing_tag| existing_tag.key() != tag.key());
        self.inner.sort();
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::wallet::entry_tag::{EntryTag, EntryTags};

    #[test]
    fn test_entry_tags_serialize() {
        let tags = EntryTags::new(vec![
            EntryTag::Tag("~a".into(), "b".into()),
            EntryTag::Tag("c".into(), "d".into()),
        ]);

        let res = serde_json::to_string(&tags).unwrap();

        assert_eq!(json!({ "~a": "b", "c": "d" }).to_string(), res);
    }

    #[test]
    fn test_entry_tags_deserialize() {
        let json = json!({"a":"b", "~c":"d"});

        let tags = EntryTags::new(vec![
            EntryTag::Tag("a".into(), "b".into()),
            EntryTag::Tag("~c".into(), "d".into()),
        ]);

        let res = serde_json::from_str(&json.to_string()).unwrap();

        assert_eq!(tags, res);
    }
}
