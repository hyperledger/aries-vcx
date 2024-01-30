use std::fmt;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

// #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
// pub enum EntryTag {
//     Tag(String, String),
// }

// impl EntryTag {
//     pub fn from_pair(pair: (String, String)) -> Self {
//         Self::Tag(pair.0, pair.1)
//     }

//     pub fn key(&self) -> &str {
//         match self {
//             Self::Tag(key, _) => key,
//         }
//     }
// }

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EntryTags {
    inner: Vec<(String, String)>,
}

impl Serialize for EntryTags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for tag in self.inner.iter() {
            map.serialize_entry(&tag.0, &tag.1)?
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
            tags.add(pair);
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
    pub fn new(inner: Vec<(String, String)>) -> Self {
        let mut items = inner;
        items.sort();

        Self { inner: items }
    }

    pub fn add(&mut self, tag: (String, String)) {
        self.inner.push(tag);
        self.inner.sort();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn into_inner(self) -> Vec<(String, String)> {
        self.inner
    }

    pub fn merge(&mut self, other: EntryTags) {
        self.inner.extend(other.into_inner());
        self.inner.sort();
    }

    pub fn remove(&mut self, tag: (String, String)) {
        self.inner.retain(|existing_tag| existing_tag.0 != tag.0);
        self.inner.sort();
    }
}

impl IntoIterator for EntryTags {
    type Item = (String, String);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<(String, String)> for EntryTags {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        let mut tags = Self::default();

        for item in iter {
            tags.add(item);
        }
        tags
    }
}

impl From<Vec<(String, String)>> for EntryTags {
    fn from(value: Vec<(String, String)>) -> Self {
        value.into_iter().fold(Self::default(), |mut memo, item| {
            memo.add(item);
            memo
        })
    }
}

impl From<EntryTags> for Vec<(String, String)> {
    fn from(value: EntryTags) -> Self {
        value.inner
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::wallet::entry_tags::EntryTags;

    #[test]
    fn test_entry_tags_serialize() {
        let tags = EntryTags::new(vec![("~a".into(), "b".into()), ("c".into(), "d".into())]);

        let res = serde_json::to_string(&tags).unwrap();

        assert_eq!(json!({ "~a": "b", "c": "d" }).to_string(), res);
    }

    #[test]
    fn test_entry_tags_deserialize() {
        let json = json!({"a":"b", "~c":"d"});

        let tags = EntryTags::new(vec![("a".into(), "b".into()), ("~c".into(), "d".into())]);

        let res = serde_json::from_str(&json.to_string()).unwrap();

        assert_eq!(tags, res);
    }
}
