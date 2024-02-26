use std::fmt;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordTag {
    key: String,
    value: String,
}

impl RecordTag {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_owned(),
            value: value.to_owned(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn into_pair(self) -> (String, String) {
        (self.key, self.value)
    }

    pub fn from_pair(pair: (String, String)) -> Self {
        Self {
            key: pair.0,
            value: pair.1,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RecordTags {
    inner: Vec<RecordTag>,
}

impl Serialize for RecordTags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for tag in self.inner.iter() {
            map.serialize_entry(tag.key(), tag.value())?
        }
        map.end()
    }
}

struct RecordTagsVisitor;

impl<'de> Visitor<'de> for RecordTagsVisitor {
    type Value = RecordTags;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map representing tags")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut tags = RecordTags::new(vec![]);

        while let Some((key, val)) = map.next_entry()? {
            tags.add(RecordTag::new(key, val));
        }

        Ok(tags)
    }
}

impl<'de> Deserialize<'de> for RecordTags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RecordTagsVisitor)
    }
}

impl RecordTags {
    pub fn new(inner: Vec<RecordTag>) -> Self {
        let mut items = inner;
        items.sort();

        Self { inner: items }
    }

    pub fn add(&mut self, tag: RecordTag) {
        self.inner.push(tag);
        self.inner.sort();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn into_inner(self) -> Vec<RecordTag> {
        self.inner
    }

    pub fn merge(&mut self, other: RecordTags) {
        self.inner.extend(other.into_inner());
        self.inner.sort();
    }

    pub fn remove(&mut self, tag: RecordTag) {
        self.inner
            .retain(|existing_tag| existing_tag.key() != tag.key());
        self.inner.sort();
    }
}

impl IntoIterator for RecordTags {
    type Item = RecordTag;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<RecordTag> for RecordTags {
    fn from_iter<T: IntoIterator<Item = RecordTag>>(iter: T) -> Self {
        let mut tags = Self::default();

        for item in iter {
            tags.add(item);
        }
        tags
    }
}

impl From<Vec<RecordTag>> for RecordTags {
    fn from(value: Vec<RecordTag>) -> Self {
        value.into_iter().fold(Self::default(), |mut memo, item| {
            memo.add(item);
            memo
        })
    }
}

impl From<RecordTags> for Vec<RecordTag> {
    fn from(value: RecordTags) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::wallet::record_tags::{RecordTag, RecordTags};

    #[test]
    fn test_record_tags_serialize() {
        let tags = RecordTags::new(vec![RecordTag::new("~a", "b"), RecordTag::new("c", "d")]);
        let res = serde_json::to_string(&tags).unwrap();

        assert_eq!(json!({ "~a": "b", "c": "d" }).to_string(), res);
    }

    #[test]
    fn test_record_tags_deserialize() {
        let json = json!({"a":"b", "~c":"d"});
        let tags = RecordTags::new(vec![RecordTag::new("a", "b"), RecordTag::new("~c", "d")]);
        let res = serde_json::from_str(&json.to_string()).unwrap();

        assert_eq!(tags, res);
    }
}
