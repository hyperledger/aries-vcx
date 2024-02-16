use std::str::FromStr;

use indy_api_types::domain::wallet::IndyRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;
use vdrtools::Locator;

use self::indy_tags::IndyTags;
use super::base_wallet::{
    record::Record, record_category::RecordCategory, search_filter::SearchFilter, BaseWallet,
};
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    WalletHandle,
};

mod indy_did_wallet;
mod indy_record_wallet;
mod indy_tags;
pub mod internal;
pub mod wallet;

#[derive(Debug)]
pub struct IndySdkWallet {
    wallet_handle: WalletHandle,
}

impl IndySdkWallet {
    pub fn new(wallet_handle: WalletHandle) -> Self {
        IndySdkWallet { wallet_handle }
    }

    pub fn get_wallet_handle(&self) -> WalletHandle {
        self.wallet_handle
    }

    #[allow(unreachable_patterns)]
    async fn search(
        &self,
        category: RecordCategory,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        let json_filter = search_filter
            .map(|filter| match filter {
                SearchFilter::JsonFilter(inner) => Ok::<String, AriesVcxCoreError>(inner),
                _ => Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidInput,
                    "filter type not supported",
                )),
            })
            .transpose()?;

        let query_json = json_filter.unwrap_or("{}".into());

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
                .map(|item| IndyRecord::deserialize(item).map_err(AriesVcxCoreError::from))
                .transpose()
        };

        let mut records = Vec::new();
        while let Some(indy_record) = next().await? {
            records.push(Record::try_from_indy_record(indy_record)?);
        }

        Ok(records)
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
pub struct IndyWalletRecord {
    id: Option<String>,
    #[serde(rename = "type")]
    record_type: Option<String>,
    pub value: Option<String>,
    tags: Option<String>,
}

impl IndyWalletRecord {
    pub fn from_record(record: Record) -> VcxCoreResult<Self> {
        let tags = if record.tags().is_empty() {
            None
        } else {
            Some(serde_json::to_string(&record.tags())?)
        };

        Ok(Self {
            id: Some(record.name().into()),
            record_type: Some(record.category().to_string()),
            value: Some(record.value().into()),
            tags,
        })
    }
}

impl Record {
    pub fn try_from_indy_record(indy_record: IndyRecord) -> VcxCoreResult<Record> {
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

#[cfg(test)]
pub mod tests {
    use super::IndySdkWallet;
    use crate::wallet::base_wallet::BaseWallet;

    pub async fn dev_setup_indy_wallet() -> Box<dyn BaseWallet> {
        use crate::wallet::indy::{wallet::create_and_open_wallet, WalletConfig};

        let config_wallet = WalletConfig {
            wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
            wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".into(),
            wallet_key_derivation: "RAW".into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();

        Box::new(IndySdkWallet::new(wallet_handle))
    }
}
