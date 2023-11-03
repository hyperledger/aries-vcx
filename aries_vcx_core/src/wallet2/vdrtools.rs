use async_trait::async_trait;
use futures::stream::BoxStream;
use indy_api_types::domain::wallet::Record;
use serde::Deserialize;
use serde_json::Value;
use vdrtools::{DidMethod, DidValue, KeyInfo, Locator, MyDidInfo};

use super::{UnpackMessageOutput, Wallet, WalletRecord};
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::indy::IndySdkWallet,
};

const WALLET_OPTIONS: &str =
    r#"{"retrieve_type": true, "retrieve_value": true, "retrieve_tags": true,}"#;

#[async_trait]
impl Wallet for IndySdkWallet {
    type Record = Record;
    type RecordIdRef = IndyWalletId;
    type SearchFilter = String;

    async fn add(&self, record: Self::Record) -> VcxCoreResult<()> {
        Locator::instance()
            .non_secret_controller
            .add_record(
                self.wallet_handle,
                record.type_,
                record.id,
                record.value,
                Some(record.tags),
            )
            .await
            .map_err(From::from)
    }

    async fn get<R>(&self, id: &Self::RecordIdRef) -> VcxCoreResult<R>
    where
        R: WalletRecord<Self>,
    {
        let record = Locator::instance()
            .non_secret_controller
            .get_record(
                self.wallet_handle,
                R::RECORD_TYPE.into(),
                id.0.to_string(),
                WALLET_OPTIONS.into(),
            )
            .await?;

        let record = serde_json::from_str(&record)?;
        R::from_wallet_record(record).map(|(_, r)| r)
    }

    async fn update(&self, update: Self::Record) -> VcxCoreResult<()> {
        let Record {
            value,
            tags,
            id,
            type_,
        } = update;

        Locator::instance()
            .non_secret_controller
            .update_record_tags(self.wallet_handle, type_.clone(), id.clone(), tags)
            .await?;

        Locator::instance()
            .non_secret_controller
            .update_record_value(self.wallet_handle, type_, id, value)
            .await?;

        Ok(())
    }

    async fn delete<R>(&self, id: &Self::RecordIdRef) -> VcxCoreResult<()>
    where
        R: WalletRecord<Self>,
    {
        Locator::instance()
            .non_secret_controller
            .delete_record(self.wallet_handle, R::RECORD_TYPE.into(), id.0.to_string())
            .await?;

        Ok(())
    }

    async fn search<'a, R>(
        &'a self,
        filter: Self::SearchFilter,
    ) -> VcxCoreResult<BoxStream<'a, VcxCoreResult<(R::RecordId, R)>>>
    where
        R: WalletRecord<Self> + Send + Sync + 'a,
    {
        let search_handle = Locator::instance()
            .non_secret_controller
            .open_search(
                self.wallet_handle,
                R::RECORD_TYPE.into(),
                filter,
                WALLET_OPTIONS.into(),
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
                .map(|item| Record::deserialize(item).map_err(AriesVcxCoreError::from))
                .transpose()
        };

        let mut records = Vec::new();
        while let Some(record) = next().await? {
            records.push(R::from_wallet_record(record));
        }

        Ok(Box::pin(futures::stream::iter(records)))
    }

    async fn create_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxCoreResult<(String, String)> {
        Locator::instance()
            .did_controller
            .create_and_store_my_did(
                self.wallet_handle,
                MyDidInfo {
                    method_name: method_name.map(|m| DidMethod(m.into())),
                    seed: seed.map(ToOwned::to_owned),
                    ..MyDidInfo::default()
                },
            )
            .await
            .map_err(From::from)
    }

    async fn did_key(&self, did: &str) -> VcxCoreResult<String> {
        Locator::instance()
            .did_controller
            .key_for_local_did(self.wallet_handle, DidValue(did.into()))
            .await
            .map_err(From::from)
    }

    async fn replace_did_key(&self, did: &str) -> VcxCoreResult<String> {
        let key = Locator::instance()
            .did_controller
            .replace_keys_start(self.wallet_handle, KeyInfo::default(), DidValue(did.into()))
            .await?;

        Locator::instance()
            .did_controller
            .replace_keys_apply(self.wallet_handle, DidValue(did.into()))
            .await?;

        Ok(key)
    }

    async fn sign(&self, verkey: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Locator::instance()
            .crypto_controller
            .crypto_sign(self.wallet_handle, verkey, msg)
            .await
            .map_err(From::from)
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        Locator::instance()
            .crypto_controller
            .crypto_verify(vk, msg, signature)
            .await
            .map_err(From::from)
    }

    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &[String],
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        if receiver_keys.is_empty() {
            return Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidLibindyParam,
                "Empty RecipientKeys has been passed",
            ));
        }

        let res = Locator::instance()
            .crypto_controller
            .pack_msg(
                msg.into(),
                receiver_keys.to_owned(),
                sender_vk.map(ToOwned::to_owned),
                self.wallet_handle,
            )
            .await?;

        Ok(res)
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput> {
        let msg = Locator::instance()
            .crypto_controller
            .unpack_msg(serde_json::from_slice(msg)?, self.wallet_handle)
            .await?;

        serde_json::from_slice(&msg).map_err(|err| {
            AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ParsingError, err.to_string())
        })
    }
}

#[repr(transparent)]
pub struct IndyWalletId(pub str);

impl AsRef<str> for IndyWalletId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
