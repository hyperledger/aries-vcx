use async_trait::async_trait;
use public_key::Key;

use super::did_data::DidData;
use crate::{errors::error::VcxCoreResult, wallet::structs_io::UnpackMessageOutput};

#[async_trait]
pub trait DidWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        kdf_method_name: Option<&str>,
    ) -> VcxCoreResult<DidData>;

    async fn key_count(&self) -> VcxCoreResult<usize>;

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
