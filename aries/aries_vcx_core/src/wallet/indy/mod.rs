use async_trait::async_trait;
use indy_api_types::domain::wallet::{default_key_derivation_method, IndyRecord};
use vdrtools::{indy_wallet::iterator::WalletIterator, Locator};

use self::indy_tag::IndyTags;
use super::base_wallet::{
    did_wallet::DidWallet,
    issuer_config::IssuerConfig,
    record::{AllRecords, PartialRecord, Record},
    BaseWallet,
};
use crate::{errors::error::VcxCoreResult, WalletHandle};

mod indy_did_wallet;
mod indy_record_wallet;
pub(crate) mod indy_tag;
mod indy_utils;
pub mod indy_wallet_record;
pub mod restore_wallet_configs;
pub mod wallet_config;
pub mod wallet_credentials;

impl From<IndyRecord> for Record {
    fn from(ir: IndyRecord) -> Self {
        Self::builder()
            .name(ir.id)
            .category(ir.type_)
            .value(ir.value)
            .tags(IndyTags::new(ir.tags).into_entry_tags())
            .build()
    }
}

impl From<Record> for IndyRecord {
    fn from(record: Record) -> Self {
        Self {
            id: record.name().into(),
            type_: record.category().into(),
            value: record.value().into(),
            tags: IndyTags::from_entry_tags(record.tags().to_owned()).into_inner(),
        }
    }
}

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
}

const WALLET_OPTIONS: &str =
    r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

const SEARCH_OPTIONS: &str = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true, "retrieveRecords": true}"#;

#[async_trait]
impl BaseWallet for IndySdkWallet {
    async fn export_wallet(&self, path: &str, backup_key: &str) -> VcxCoreResult<()> {
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

    async fn close_wallet(&self) -> VcxCoreResult<()> {
        Locator::instance()
            .wallet_controller
            .close(self.wallet_handle)
            .await?;

        Ok(())
    }

    async fn configure_issuer(&self, key_seed: &str) -> VcxCoreResult<IssuerConfig> {
        let did_data = self.create_and_store_my_did(Some(key_seed), None).await?;

        Ok(IssuerConfig {
            institution_did: did_data.did().to_string(),
        })
    }

    async fn all(&self) -> VcxCoreResult<Box<dyn AllRecords + Send>> {
        let all = Locator::instance()
            .wallet_controller
            .get_all(self.get_wallet_handle())
            .await?;

        Ok(Box::new(AllIndyRecords::new(all)))
    }
}

pub struct AllIndyRecords {
    iterator: WalletIterator,
}

impl AllIndyRecords {
    pub fn new(iterator: WalletIterator) -> Self {
        Self { iterator }
    }
}

#[async_trait]
impl AllRecords for AllIndyRecords {
    fn total_count(&self) -> VcxCoreResult<Option<usize>> {
        Ok(self.iterator.get_total_count()?)
    }

    async fn next(&mut self) -> VcxCoreResult<Option<PartialRecord>> {
        let item = self.iterator.next().await?;

        Ok(item.map(PartialRecord::from_wallet_record))
    }
}

#[cfg(test)]
pub mod tests {

    use std::sync::Arc;

    use crate::wallet::{
        base_wallet::{BaseWallet, ManageWallet},
        indy::wallet_config::WalletConfig,
    };

    pub async fn dev_setup_indy_wallet() -> Arc<dyn BaseWallet> {
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

        config_wallet.create_wallet().await.unwrap();
        config_wallet.open_wallet().await.unwrap()
    }
}
