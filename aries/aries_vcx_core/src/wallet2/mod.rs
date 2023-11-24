use std::collections::HashMap;

use aries_askar::entry::{Entry, EntryKind, EntryTag as AskarEntryTag, TagFilter};
use async_trait::async_trait;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

#[cfg(feature = "vdrtools_wallet")]
use indy_api_types::domain::wallet::Record as IndyRecord;

#[cfg(feature = "vdrtools_wallet")]
pub mod indy_wallet;

#[cfg(feature = "askar_wallet")]
pub mod askar_wallet;

#[derive(Clone, Default)]
pub enum RngMethod {
    #[default]
    RandomDet,
    Bls,
}

impl From<RngMethod> for Option<&str> {
    fn from(value: RngMethod) -> Self {
        match value {
            RngMethod::RandomDet => None,
            RngMethod::Bls => Some("bls_keygen"),
        }
    }
}
pub enum SigType {
    EdDSA,
    ES256,
    ES256K,
    ES384,
}

impl From<SigType> for &str {
    fn from(value: SigType) -> Self {
        match value {
            SigType::EdDSA => "eddsa",
            SigType::ES256 => "es256",
            SigType::ES256K => "es256k",
            SigType::ES384 => "es384",
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryTag {
    Encrypted(String, String),
    Plaintext(String, String),
}

#[cfg(feature = "askar_wallet")]
impl From<AskarEntryTag> for EntryTag {
    fn from(value: AskarEntryTag) -> Self {
        match value {
            AskarEntryTag::Encrypted(key, val) => Self::Encrypted(key, val),
            AskarEntryTag::Plaintext(key, val) => Self::Plaintext(key, val),
        }
    }
}

#[cfg(feature = "askar_wallet")]
impl From<EntryTag> for AskarEntryTag {
    fn from(value: EntryTag) -> Self {
        match value {
            EntryTag::Encrypted(key, val) => Self::Encrypted(key, val),
            EntryTag::Plaintext(key, val) => Self::Plaintext(key, val),
        }
    }
}

#[derive(Debug, Default, Clone, Builder)]
pub struct Record {
    pub category: String,
    pub name: String,
    pub value: String,
    #[builder(default = "vec![]")]
    pub tags: Vec<EntryTag>,
}

#[cfg(feature = "vdrtools_wallet")]
impl From<IndyRecord> for Record {
    fn from(ir: IndyRecord) -> Self {
        let tags = ir
            .tags
            .into_iter()
            .map(|(key, value)| EntryTag::Plaintext(key, value))
            .collect();
        Self {
            name: ir.id,
            category: ir.type_,
            value: ir.value,
            tags,
        }
    }
}

#[cfg(feature = "vdrtools_wallet")]
impl From<Record> for IndyRecord {
    fn from(record: Record) -> Self {
        let tags = record
            .tags
            .into_iter()
            .fold(HashMap::new(), |mut memo, item| {
                match item {
                    EntryTag::Encrypted(key, val) => memo.insert(key, val),
                    EntryTag::Plaintext(key, val) => memo.insert(format!("~{}", key), val),
                };
                memo
            });
        Self {
            id: record.name,
            type_: record.category,
            value: record.value,
            tags,
        }
    }
}

#[cfg(feature = "askar_wallet")]
impl TryFrom<Entry> for Record {
    type Error = AriesVcxCoreError;

    fn try_from(entry: Entry) -> Result<Self, Self::Error> {
        let string_value = std::str::from_utf8(&entry.value).map_err(|err| {
            AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletUnexpected, err)
        })?;

        Ok(Self {
            category: entry.category,
            name: entry.name,
            value: string_value.into(),
            tags: entry.tags.into_iter().map(From::from).collect(),
        })
    }
}

#[cfg(feature = "askar_wallet")]
impl From<Record> for Entry {
    fn from(record: Record) -> Self {
        Self {
            category: record.category,
            name: record.name,
            value: record.value.into(),
            kind: EntryKind::Item,
            tags: record.tags.into_iter().map(From::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DidData {
    did: String,
    verkey: String,
}

pub enum SearchFilter {
    TagFilter(TagFilter),
    JsonFilter(String),
}

#[async_trait]
pub trait BaseWallet2: RecordWallet + DidWallet {}

#[async_trait]
pub trait DidWallet {
    async fn create_and_store_my_did(
        &self,
        seed: &str,
        method_name: Option<&str>,
    ) -> VcxCoreResult<DidData>;

    async fn did_key(&self, name: &str) -> VcxCoreResult<String>;

    async fn replace_did_key(&self, did: &str, seed: &str) -> VcxCoreResult<String>;

    async fn sign(&self, key: &str, msg: &[u8], sig_type: SigType) -> VcxCoreResult<Vec<u8>>;

    async fn verify(
        &self,
        key: &str,
        msg: &[u8],
        signature: &[u8],
        sig_type: SigType,
    ) -> VcxCoreResult<bool>;
}

#[async_trait]
pub trait RecordWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()>;

    async fn get_record(&self, name: &str, category: &str) -> VcxCoreResult<Record>;

    async fn update_record(&self, record: Record) -> VcxCoreResult<()>;

    async fn delete_record(&self, name: &str, category: &str) -> VcxCoreResult<()>;

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>>;
}
