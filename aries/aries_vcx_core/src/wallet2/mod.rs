use std::collections::HashMap;

use async_trait::async_trait;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::errors::error::VcxCoreResult;

#[cfg(feature = "vdrtools_wallet")]
use indy_api_types::domain::wallet::Record as IndyRecord;

#[cfg(feature = "vdrtools_wallet")]
pub mod indy_wallet;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct DidData {
    did: String,
    verkey: String,
}

pub enum SearchFilter {
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
