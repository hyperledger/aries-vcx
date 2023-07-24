use std::collections::HashMap;

use async_trait::async_trait;

use crate::{indy, WalletHandle};
use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use crate::utils::async_fn_iterator::AsyncFnIterator;
use crate::wallet::base_wallet::BaseWallet;
use crate::wallet::indy::{IndySdkWallet, IndyWalletRecordIterator, internal, WalletRecord};

#[allow(unused_variables)]
#[async_trait]
impl BaseWallet for IndySdkWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxCoreResult<(String, String)> {
        indy::wallet::create_and_store_my_did(self.wallet_handle, seed, method_name).await
    }

    async fn key_for_local_did(&self, did: &str) -> VcxCoreResult<String> {
        indy::wallet::get_verkey_from_wallet(self.wallet_handle, did).await
    }

    async fn replace_did_keys_start(&self, target_did: &str) -> VcxCoreResult<String> {
        indy::wallet::libindy_replace_keys_start(self.wallet_handle, target_did).await
    }

    async fn replace_did_keys_apply(&self, target_did: &str) -> VcxCoreResult<()> {
        indy::wallet::libindy_replace_keys_apply(self.wallet_handle, target_did).await
    }

    async fn add_wallet_record(
        &self,
        xtype: &str,
        id: &str,
        value: &str,
        tags: Option<HashMap<String, String>>,
    ) -> VcxCoreResult<()> {
        let res = tags.map(|x| serde_json::to_string(&x)).transpose()?.to_owned();
        let tags_json = res.as_deref();
        internal::add_wallet_record(self.wallet_handle, xtype, id, value, tags_json).await
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options: &str) -> VcxCoreResult<String> {
        internal::get_wallet_record(self.wallet_handle, xtype, id, options).await
    }

    async fn get_wallet_record_value(&self, xtype: &str, id: &str) -> VcxCoreResult<String> {
        let options = r#"{"retrieveType": false, "retrieveValue": true, "retrieveTags": false}"#;

        let str_record = self.get_wallet_record(xtype, id, options).await?;
        let wallet_record: WalletRecord = serde_json::from_str(&str_record)?;
        wallet_record.value.ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::WalletRecordNotFound,
                "The wallet record does not have a value",
            )
        })
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxCoreResult<()> {
        internal::delete_wallet_record(self.wallet_handle, xtype, id).await
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxCoreResult<()> {
        internal::update_wallet_record_value(self.wallet_handle, xtype, id, value).await
    }

    async fn update_wallet_record_tags(
        &self,
        xtype: &str,
        id: &str,
        tags: HashMap<String, String>,
    ) -> VcxCoreResult<()> {
        let tags_json = serde_json::to_string(&tags)?;
        internal::update_wallet_record_tags(self.wallet_handle, xtype, id, &tags_json).await
    }

    async fn add_wallet_record_tags(&self, xtype: &str, id: &str, tags: HashMap<String, String>) -> VcxCoreResult<()> {
        let tags_json = serde_json::to_string(&tags)?;
        internal::add_wallet_record_tags(self.wallet_handle, xtype, id, &tags_json).await
    }

    async fn delete_wallet_record_tags(&self, xtype: &str, id: &str, tag_names: &str) -> VcxCoreResult<()> {
        internal::delete_wallet_record_tags(self.wallet_handle, xtype, id, tag_names).await
    }

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxCoreResult<Box<dyn AsyncFnIterator<Item=VcxCoreResult<String>>>> {
        let search = internal::open_search_wallet(self.wallet_handle, xtype, query, options).await?;
        let iter = IndyWalletRecordIterator::new(self.wallet_handle, search);

        Ok(Box::new(iter))
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        indy::signing::sign(self.wallet_handle, my_vk, msg).await
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        indy::signing::verify(vk, msg, signature).await
    }

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        indy::signing::pack_message(self.wallet_handle, sender_vk, receiver_keys, msg).await
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        indy::signing::unpack_message(self.wallet_handle, msg).await
    }

    fn get_wallet_handle(&self) -> WalletHandle {
        self.wallet_handle
    }
}
