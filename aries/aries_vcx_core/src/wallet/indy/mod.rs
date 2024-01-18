use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    base_wallet::{BaseWallet, Record},
    entry_tag::{EntryTag, EntryTags},
};
use crate::{errors::error::VcxCoreResult, WalletHandle};

pub mod indy_did_wallet;
pub mod indy_record_wallet;
pub mod internal;
pub mod signing;
pub mod wallet;
pub mod wallet_non_secrets;

#[derive(Debug)]
pub struct IndySdkWallet {
    pub wallet_handle: WalletHandle,
}

impl IndySdkWallet {
    pub fn new(wallet_handle: WalletHandle) -> Self {
        IndySdkWallet { wallet_handle }
    }

    pub fn get_wallet_handle(&self) -> WalletHandle {
        self.wallet_handle
    }
}

#[derive(Clone, Debug, TypedBuilder, Serialize, Deserialize)]
#[builder(field_defaults(default))]
pub struct WalletConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_key_derivation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub wallet_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub storage_config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub storage_credentials: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(strip_option))]
    pub rekey_derivation_method: Option<String>,
}

#[derive(Clone, Debug, TypedBuilder, Serialize, Deserialize)]
#[builder(field_defaults(default))]
pub struct IssuerConfig {
    pub institution_did: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WalletCredentials {
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_credentials: Option<serde_json::Value>,
    key_derivation_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rekey_derivation_method: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletRecord {
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    pub value: Option<String>,
    tags: Option<String>,
}

impl WalletRecord {
    pub fn from_record(record: Record) -> VcxCoreResult<Self> {
        let tags = if record.tags().is_empty() {
            None
        } else {
            Some(serde_json::to_string(&record.tags())?)
        };

        Ok(Self {
            id: Some(record.name().into()),
            record_type: Some(record.category().into()),
            value: Some(record.value().into()),
            tags,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreWalletConfigs {
    pub wallet_name: String,
    pub wallet_key: String,
    pub exported_wallet_path: String,
    pub backup_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_key_derivation: Option<String>,
}

const WALLET_OPTIONS: &str =
    r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

const SEARCH_OPTIONS: &str = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true, "retrieveRecords": true}"#;

impl BaseWallet for IndySdkWallet {}

pub struct IndyTag((String, String));

impl IndyTag {
    pub fn new(pair: (String, String)) -> Self {
        Self(pair)
    }

    pub fn to_inner(self) -> (String, String) {
        self.0
    }

    pub fn to_entry_tag(self) -> EntryTag {
        let inner = self.to_inner();

        if inner.0.starts_with('~') {
            EntryTag::Plaintext(inner.0.trim_start_matches('~').into(), inner.1)
        } else {
            EntryTag::Encrypted(inner.0, inner.1)
        }
    }

    pub fn from_entry_tag(tag: EntryTag) -> Self {
        match tag {
            EntryTag::Encrypted(key, val) => Self((key, val)),
            EntryTag::Plaintext(key, val) => Self((format!("~{}", key), val)),
        }
    }
}

pub struct IndyTags(HashMap<String, String>);

impl IndyTags {
    pub fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    pub fn to_inner(self) -> HashMap<String, String> {
        self.0
    }

    pub fn from_entry_tags(tags: EntryTags) -> Self {
        let mut map = HashMap::new();
        let tags_vec: Vec<_> = tags
            .into_iter()
            .map(|tag| IndyTag::from_entry_tag(tag).to_inner())
            .collect();
        map.extend(tags_vec);
        Self(map)
    }

    pub fn to_entry_tags(self) -> EntryTags {
        let mut items: Vec<EntryTag> = self
            .0
            .into_iter()
            .map(|pair| IndyTag::new(pair).to_entry_tag())
            .collect();
        items.sort();

        EntryTags::new(items)
    }
}
