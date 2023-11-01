use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait Wallet {
    type Record: Send + Sync;
    type RecordIdRef<'a>: Send + Sync;
    type RecordUpdate<'a>: Send + Sync;
    type SearchFilter<'a>: Send + Sync;

    async fn add(&self, id: Self::RecordIdRef<'_>, record: Self::Record) -> VcxCoreResult<()>;

    async fn get(&self, id: Self::RecordIdRef<'_>) -> VcxCoreResult<Self::Record>;

    async fn search(
        &self,
        filter: Self::SearchFilter<'_>,
    ) -> VcxCoreResult<BoxStream<'static, VcxCoreResult<Self::Record>>>;

    async fn update(
        &self,
        id: Self::RecordIdRef<'_>,
        update: Self::RecordUpdate<'_>,
    ) -> VcxCoreResult<()>;

    async fn delete<R>(&self, id: Self::RecordIdRef<'_>) -> VcxCoreResult<()>
    where
        R: WalletRecord<Self>;

    async fn create_did(
        &self,
        seed: Option<&str>,
        kdf_method_name: Option<&str>,
    ) -> VcxCoreResult<(String, String)>;

    async fn did_key(&self, did: &str) -> VcxCoreResult<String>;

    async fn replace_did_key(&self, target_did: &str) -> VcxCoreResult<String>;

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>>;

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool>;

    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput>;
}

pub trait WalletRecord<W: Wallet + ?Sized> {
    const RECORD_TYPE: &'static str;

    type RecordParams<'a>;

    fn into_wallet_record(self, params: Self::RecordParams<'_>) -> VcxCoreResult<W::Record>;

    fn from_wallet_record(record: W::Record) -> VcxCoreResult<Self>
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UnpackMessageOutput {
    pub message: String,
    pub recipient_verkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_verkey: Option<String>,
}
