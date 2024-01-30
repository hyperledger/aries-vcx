use std::sync::Arc;

use async_trait::async_trait;
use public_key::Key;

use crate::{errors::error::VcxCoreResult, wallet::structs_io::UnpackMessageOutput};

use super::{did_data::DidData, BaseWallet};

#[async_trait]
pub trait DidWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        kdf_method_name: Option<&str>,
    ) -> VcxCoreResult<DidData>;

    async fn key_for_did(&self, did: &str) -> VcxCoreResult<Key>;

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxCoreResult<Key>;

    async fn replace_did_key_apply(&self, did: &str) -> VcxCoreResult<()>;

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxCoreResult<Vec<u8>>;

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool>;

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput>;
}

#[async_trait]
impl DidWallet for Arc<dyn BaseWallet> {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        did_method_name: Option<&str>,
    ) -> VcxCoreResult<DidData> {
        self.as_ref()
            .create_and_store_my_did(seed, did_method_name)
            .await
    }

    async fn key_for_did(&self, did: &str) -> VcxCoreResult<Key> {
        self.as_ref().key_for_did(did).await
    }

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxCoreResult<Key> {
        self.as_ref().replace_did_key_start(did, seed).await
    }

    async fn replace_did_key_apply(&self, did: &str) -> VcxCoreResult<()> {
        self.as_ref().replace_did_key_apply(did).await
    }

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        self.as_ref().sign(key, msg).await
    }

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        self.as_ref().verify(key, msg, signature).await
    }

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        self.as_ref()
            .pack_message(sender_vk, receiver_keys, msg)
            .await
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput> {
        self.as_ref().unpack_message(msg).await
    }
}
