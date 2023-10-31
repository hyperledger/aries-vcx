use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait Wallet {
    type Record: Send + Sync;
    type RecordUpdate<'a>: Send + Sync;
    type SearchFilter<'a>: Send + Sync;

    async fn add<R>(&self, record: R) -> VcxCoreResult<()>
    where
        R: WalletRecord<Self> + Send,
    {
        let record = record.into_wallet_record()?;
        self.add_record(record).await
    }

    async fn add_record(&self, record: Self::Record) -> VcxCoreResult<()>;

    async fn get<R>(&self, id: &str) -> VcxCoreResult<R::Value>
    where
        R: WalletRecord<Self>,
    {
        let record = self.get_record(id).await?;
        R::from_wallet_record(record)
    }

    async fn get_record(&self, id: &str) -> VcxCoreResult<Self::Record>;

    async fn search(
        &self,
        filter: Self::SearchFilter<'_>,
    ) -> VcxCoreResult<BoxStream<'static, VcxCoreResult<Self::Record>>>;

    async fn update(&self, update: Self::RecordUpdate<'_>) -> VcxCoreResult<()>;

    async fn delete<R>(&self, id: &str) -> VcxCoreResult<()>
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

    type Value;

    fn into_wallet_record(self) -> VcxCoreResult<W::Record>;

    fn from_wallet_record(record: W::Record) -> VcxCoreResult<Self::Value>
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
