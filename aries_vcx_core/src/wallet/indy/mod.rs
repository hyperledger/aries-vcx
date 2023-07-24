pub mod internal;
pub mod indy_wallet;

use std::collections::HashMap;
use std::thread;

use async_trait::async_trait;
use futures::executor::block_on;
use serde_json::Value;
use serde::{Deserialize, Serialize};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use crate::{indy, utils::{async_fn_iterator::AsyncFnIterator, json::TryGetIndex}, wallet};
use crate::{SearchHandle, WalletHandle};

use super::base_wallet::BaseWallet;

#[derive(Debug)]
pub struct IndySdkWallet {
    pub wallet_handle: WalletHandle,
}

impl IndySdkWallet {
    pub fn new(wallet_handle: WalletHandle) -> Self {
        IndySdkWallet { wallet_handle }
    }
}

struct IndyWalletRecordIterator {
    wallet_handle: WalletHandle,
    search_handle: SearchHandle,
}

impl IndyWalletRecordIterator {
    fn new(wallet_handle: WalletHandle, search_handle: SearchHandle) -> Self {
        IndyWalletRecordIterator {
            wallet_handle,
            search_handle,
        }
    }

    async fn fetch_next_records(&self) -> VcxCoreResult<Option<String>> {
        let indy_res_json = internal::fetch_next_records_wallet(self.wallet_handle, self.search_handle, 1).await?;

        let indy_res: Value = serde_json::from_str(&indy_res_json)?;

        let records = (&indy_res).try_get("records")?;

        let item: Option<VcxCoreResult<String>> = records
            .as_array()
            .and_then(|arr| arr.first())
            .map(|item| serde_json::to_string(item).map_err(AriesVcxCoreError::from));

        item.transpose()
    }
}

/// Implementation of a generic [AsyncFnIterator] iterator for indy/vdrtools wallet record iteration.
/// Wraps over the vdrtools record [SearchHandle] functionality
#[async_trait]
impl AsyncFnIterator for IndyWalletRecordIterator {
    type Item = VcxCoreResult<String>;

    async fn next(&mut self) -> Option<Self::Item> {
        let records = self.fetch_next_records().await;
        records.transpose()
    }
}

impl Drop for IndyWalletRecordIterator {
    fn drop(&mut self) {
        let search_handle = self.search_handle;

        thread::spawn(move || {
            block_on(async {
                internal::close_search_wallet(search_handle).await.ok();
            });
        });
    }
}

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
pub struct WalletConfig {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_key_derivation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_credentials: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_derivation_method: Option<String>,
}

#[derive(Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into, strip_option), default)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreWalletConfigs {
    pub wallet_name: String,
    pub wallet_key: String,
    pub exported_wallet_path: String,
    pub backup_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_key_derivation: Option<String>,
}
