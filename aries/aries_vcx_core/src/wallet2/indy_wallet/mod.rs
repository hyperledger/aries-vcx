use std::collections::HashMap;

use super::{
    entry_tag::{EntryTag, EntryTags},
    BaseWallet2,
};
use crate::wallet::indy::IndySdkWallet;

pub mod indy_did_wallet;
pub mod indy_record_wallet;

const WALLET_OPTIONS: &str =
    r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

const SEARCH_OPTIONS: &str = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true, "retrieveRecords": true}"#;

impl BaseWallet2 for IndySdkWallet {}

impl From<EntryTag> for (String, String) {
    fn from(value: EntryTag) -> Self {
        match value {
            EntryTag::Encrypted(key, val) => (key, val),
            EntryTag::Plaintext(key, val) => (format!("~{}", key), val),
        }
    }
}

impl From<(String, String)> for EntryTag {
    fn from(value: (String, String)) -> Self {
        if value.0.starts_with('~') {
            EntryTag::Plaintext(value.0.trim_start_matches('~').into(), value.1)
        } else {
            EntryTag::Encrypted(value.0, value.1)
        }
    }
}

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

impl From<HashMap<String, String>> for EntryTags {
    fn from(value: HashMap<String, String>) -> Self {
        let mut items: Vec<EntryTag> = value
            .into_iter()
            .map(|(key, value)| (key, value))
            .map(From::from)
            .collect();

        items.sort();

        Self::new(items)
    }
}
