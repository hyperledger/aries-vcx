use std::fmt;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum EntryTag {
    Encrypted(String, String),
    Plaintext(String, String),
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
                EntryTag::Encrypted(key, val) => map.serialize_entry(&key, &val)?,
                EntryTag::Plaintext(key, val) => map.serialize_entry(&format!("~{}", key), &val)?,
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
            tags.add(pair_into_entry_tag(pair));
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
        Self { inner }
    }

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

pub fn pair_into_entry_tag(pair: (String, String)) -> EntryTag {
    if pair.0.starts_with('~') {
        EntryTag::Plaintext(pair.0.trim_start_matches('~').into(), pair.1)
    } else {
        EntryTag::Encrypted(pair.0, pair.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::entry_tag::{EntryTag, EntryTags};

    #[test]
    fn should_serialize_entry_tags() {
        let tags = EntryTags::new(vec![
            EntryTag::Plaintext("a".into(), "b".into()),
            EntryTag::Encrypted("c".into(), "d".into()),
        ]);

        let res = serde_json::to_string(&tags).unwrap();

        assert_eq!("{\"~a\":\"b\",\"c\":\"d\"}", res);
    }

    #[test]
    fn shoud_deserialize_entry_tags() {
        let json = "{\"a\":\"b\",\"~c\":\"d\"}";

        let tags = EntryTags::new(vec![
            EntryTag::Encrypted("a".into(), "b".into()),
            EntryTag::Plaintext("c".into(), "d".into()),
        ]);

        let res = serde_json::from_str(json).unwrap();

        assert_eq!(tags, res);
    }
}
