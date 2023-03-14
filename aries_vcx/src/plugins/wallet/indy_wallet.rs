use std::thread;

use async_trait::async_trait;
use futures::executor::block_on;
use serde_json::Value;
use vdrtools::{SearchHandle, WalletHandle};

use super::base_wallet::BaseWallet;
use crate::{
    errors::error::{AriesVcxError, VcxResult},
    indy::{self},
    utils::{async_fn_iterator::AsyncFnIterator, json::TryGetIndex},
};

#[derive(Debug)]
pub struct IndySdkWallet {
    wallet_handle: WalletHandle,
}

impl IndySdkWallet {
    pub fn new(wallet_handle: WalletHandle) -> Self {
        IndySdkWallet { wallet_handle }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl BaseWallet for IndySdkWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxResult<(String, String)> {
        indy::keys::create_and_store_my_did(self.wallet_handle, seed, method_name).await
    }

    async fn key_for_local_did(&self, did: &str) -> VcxResult<String> {
        indy::keys::get_verkey_from_wallet(self.wallet_handle, did).await
    }

    async fn replace_did_keys_start(&self, target_did: &str) -> VcxResult<String> {
        indy::keys::libindy_replace_keys_start(self.wallet_handle, target_did).await
    }

    async fn replace_did_keys_apply(&self, target_did: &str) -> VcxResult<()> {
        indy::keys::libindy_replace_keys_apply(self.wallet_handle, target_did).await
    }

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags_json: Option<&str>) -> VcxResult<()> {
        indy::wallet::add_wallet_record(self.wallet_handle, xtype, id, value, tags_json).await
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options_json: &str) -> VcxResult<String> {
        indy::wallet::get_wallet_record(self.wallet_handle, xtype, id, options_json).await
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()> {
        indy::wallet::delete_wallet_record(self.wallet_handle, xtype, id).await
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()> {
        indy::wallet::update_wallet_record_value(self.wallet_handle, xtype, id, value).await
    }

    async fn update_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()> {
        indy::wallet::update_wallet_record_tags(self.wallet_handle, xtype, id, tags_json).await
    }

    async fn add_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()> {
        indy::wallet::add_wallet_record_tags(self.wallet_handle, xtype, id, tags_json).await
    }

    async fn delete_wallet_record_tags(&self, xtype: &str, id: &str, tag_names: &str) -> VcxResult<()> {
        indy::wallet::delete_wallet_record_tags(self.wallet_handle, xtype, id, tag_names).await
    }

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxResult<Box<dyn AsyncFnIterator<Item = VcxResult<String>>>> {
        let search = indy::wallet::open_search_wallet(self.wallet_handle, xtype, query, options).await?;
        let iter = IndyWalletRecordIterator::new(self.wallet_handle, search);

        Ok(Box::new(iter))
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        indy::signing::sign(self.wallet_handle, my_vk, msg).await
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
        indy::signing::verify(vk, msg, signature).await
    }

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        indy::signing::pack_message(self.wallet_handle, sender_vk, receiver_keys, msg).await
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxResult<Vec<u8>> {
        indy::signing::unpack_message(self.wallet_handle, msg).await
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

    async fn fetch_next_records(&self) -> VcxResult<Option<String>> {
        let indy_res_json = indy::wallet::fetch_next_records_wallet(self.wallet_handle, self.search_handle, 1).await?;

        let indy_res: Value = serde_json::from_str(&indy_res_json)?;

        let records = (&indy_res).try_get("records")?;

        let item: Option<VcxResult<String>> = records
            .as_array()
            .and_then(|arr| arr.first())
            .map(|item| serde_json::to_string(item).map_err(AriesVcxError::from));

        item.transpose()
    }
}

/// Implementation of a generic [AsyncFnIterator] iterator for indy/vdrtools wallet record
/// iteration. Wraps over the vdrtools record [SearchHandle] functionality
#[async_trait]
impl AsyncFnIterator for IndyWalletRecordIterator {
    type Item = VcxResult<String>;

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
                indy::wallet::close_search_wallet(search_handle).await.ok();
            });
        });
    }
}
