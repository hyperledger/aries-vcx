use aries_askar::entry::EntryTag;

use crate::wallet::record_tags::{RecordTag, RecordTags};

impl From<EntryTag> for RecordTag {
    fn from(askar_tag: EntryTag) -> Self {
        match askar_tag {
            EntryTag::Encrypted(key, val) => RecordTag::new(&key, &val),
            EntryTag::Plaintext(key, val) => RecordTag::new(&format!("~{}", key), &val),
        }
    }
}

impl From<RecordTag> for EntryTag {
    fn from(entry_tag: RecordTag) -> Self {
        if entry_tag.key().starts_with('~') {
            Self::Plaintext(
                entry_tag.key().to_string().trim_start_matches('~').into(),
                entry_tag.value().into(),
            )
        } else {
            Self::Encrypted(entry_tag.key().into(), entry_tag.value().into())
        }
    }
}

impl From<RecordTags> for Vec<EntryTag> {
    fn from(tags: RecordTags) -> Self {
        let tags_vec: Vec<RecordTag> = tags.into();
        tags_vec.into_iter().map(Into::into).collect()
    }
}

impl From<Vec<EntryTag>> for RecordTags {
    fn from(askar_tags: Vec<EntryTag>) -> Self {
        askar_tags.into_iter().map(Into::into).collect()
    }
}
