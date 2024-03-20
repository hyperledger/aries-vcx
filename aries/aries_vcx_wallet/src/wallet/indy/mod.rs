use std::str::FromStr;

use async_trait::async_trait;
use indy_api_types::{
    domain::wallet::{default_key_derivation_method, IndyRecord},
    errors::IndyErrorKind,
};
use log::warn;
use serde::Deserialize;
use serde_json::Value;
use vdrtools::{Locator, WalletHandle};

use self::{indy_tags::IndyTags, indy_wallet_config::IndyWalletConfig};
use super::{
    base_wallet::{
        key_value::KeyValue, record::Record, record_category::RecordCategory,
        record_wallet::RecordWallet, BaseWallet,
    },
    record_tags::RecordTags,
};
use crate::errors::error::{VcxWalletError, VcxWalletResult};

mod all_indy_records;
mod indy_did_wallet;
pub mod indy_import_config;
mod indy_record_wallet;
mod indy_tags;
mod indy_utils;
pub mod indy_wallet_config;
pub mod indy_wallet_record;
mod partial_record;

impl Record {
    pub fn try_from_indy_record(indy_record: IndyRecord) -> VcxWalletResult<Record> {
        Ok(Record::builder()
            .name(indy_record.id)
            .category(RecordCategory::from_str(&indy_record.type_)?)
            .value(indy_record.value)
            .tags(IndyTags::new(indy_record.tags).into_record_tags())
            .build())
    }
}

impl From<Record> for IndyRecord {
    fn from(record: Record) -> Self {
        Self {
            id: record.name().into(),
            type_: record.category().to_string(),
            value: record.value().into(),
            tags: IndyTags::from_record_tags(record.tags().to_owned()).into_inner(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndySdkWallet {
    wallet_handle: WalletHandle,
}

impl IndySdkWallet {
    pub fn new(wallet_handle: WalletHandle) -> Self {
        IndySdkWallet { wallet_handle }
    }

    pub async fn create(wallet_config: &IndyWalletConfig) -> VcxWalletResult<Self> {
        let (config, creds) = wallet_config.to_config_and_creds()?;

        let res = Locator::instance()
            .wallet_controller
            .create(config, creds)
            .await;

        match res {
            Ok(()) => Self::open(wallet_config).await,

            Err(err) if err.kind() == IndyErrorKind::WalletAlreadyExists => {
                warn!(
                    "wallet \"{}\" already exists. skipping creation",
                    wallet_config.wallet_name
                );
                Self::open(wallet_config).await
            }

            Err(err) => Err(VcxWalletError::create_wallet_error(err)),
        }
    }

    async fn open(wallet_config: &IndyWalletConfig) -> VcxWalletResult<Self> {
        let (config, creds) = wallet_config.to_config_and_creds()?;

        let handle = Locator::instance()
            .wallet_controller
            .open(config, creds)
            .await?;

        Ok(Self::new(handle))
    }

    pub fn get_wallet_handle(&self) -> WalletHandle {
        self.wallet_handle
    }

    #[allow(unreachable_patterns)]
    async fn search(
        &self,
        category: RecordCategory,
        search_filter: Option<String>,
    ) -> VcxWalletResult<Vec<Record>> {
        let query_json = search_filter.unwrap_or("{}".into());

        let search_handle = Locator::instance()
            .non_secret_controller
            .open_search(
                self.wallet_handle,
                category.to_string(),
                query_json,
                SEARCH_OPTIONS.into(),
            )
            .await?;

        let next = || async {
            let record = Locator::instance()
                .non_secret_controller
                .fetch_search_next_records(self.wallet_handle, search_handle, 1)
                .await?;

            let indy_res: Value = serde_json::from_str(&record)?;

            indy_res
                .get("records")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .map(|item| IndyRecord::deserialize(item).map_err(VcxWalletError::from))
                .transpose()
        };

        let mut records = Vec::new();
        while let Some(indy_record) = next().await? {
            records.push(Record::try_from_indy_record(indy_record)?);
        }

        Ok(records)
    }
}

const WALLET_OPTIONS: &str =
    r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

const SEARCH_OPTIONS: &str = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true, "retrieveRecords": true}"#;

#[async_trait]
impl BaseWallet for IndySdkWallet {
    async fn export_wallet(&self, path: &str, backup_key: &str) -> VcxWalletResult<()> {
        Locator::instance()
            .wallet_controller
            .export(
                self.wallet_handle,
                vdrtools::types::domain::wallet::ExportConfig {
                    key: backup_key.into(),
                    path: path.into(),

                    key_derivation_method: default_key_derivation_method(),
                },
            )
            .await?;

        Ok(())
    }

    async fn close_wallet(&self) -> VcxWalletResult<()> {
        Locator::instance()
            .wallet_controller
            .close(self.wallet_handle)
            .await?;

        Ok(())
    }

    async fn create_key(
        &self,
        name: &str,
        value: KeyValue,
        tags: &RecordTags,
    ) -> VcxWalletResult<()> {
        let value = serde_json::to_string(&value)?;
        let record = Record::builder()
            .name(name.into())
            .value(value)
            .category(RecordCategory::Key)
            .tags(tags.clone())
            .build();

        Ok(self.add_record(record).await?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::IndySdkWallet;
    use crate::wallet::{base_wallet::ManageWallet, indy::indy_wallet_config::IndyWalletConfig};

    pub async fn dev_setup_indy_wallet() -> IndySdkWallet {
        let config_wallet = IndyWalletConfig {
            wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
            wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".into(),
            wallet_key_derivation: "RAW".into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };

        config_wallet.create_wallet().await.unwrap()
    }
}
