use async_trait::async_trait;
use indy_api_types::domain::wallet::IndyRecord;
use serde::Deserialize;
use serde_json::Value;
use vdrtools::Locator;

use super::{indy_tags::IndyTags, SEARCH_OPTIONS, WALLET_OPTIONS};
use crate::{
    errors::error::{AriesVcxCoreError, VcxCoreResult},
    wallet::{
        base_wallet::{record::Record, search_filter::SearchFilter, RecordWallet},
        entry_tags::EntryTags,
        indy::IndySdkWallet,
    },
};

#[async_trait]
impl RecordWallet for IndySdkWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        let tags_map = if record.tags().is_empty() {
            None
        } else {
            Some(IndyTags::from_entry_tags(record.tags().clone()).into_inner())
        };

        Ok(Locator::instance()
            .non_secret_controller
            .add_record(
                self.wallet_handle,
                record.category().into(),
                record.name().into(),
                record.value().into(),
                tags_map,
            )
            .await?)
    }

    async fn get_record(&self, category: &str, name: &str) -> VcxCoreResult<Record> {
        let res = Locator::instance()
            .non_secret_controller
            .get_record(
                self.wallet_handle,
                category.into(),
                name.into(),
                WALLET_OPTIONS.into(),
            )
            .await?;

        let indy_record: IndyRecord = serde_json::from_str(&res)?;

        Ok(indy_record.into())
    }

    async fn update_record_tags(
        &self,
        category: &str,
        name: &str,
        new_tags: EntryTags,
    ) -> VcxCoreResult<()> {
        Ok(Locator::instance()
            .non_secret_controller
            .update_record_tags(
                self.wallet_handle,
                category.into(),
                name.into(),
                IndyTags::from_entry_tags(new_tags).into_inner(),
            )
            .await?)
    }

    async fn update_record_value(
        &self,
        category: &str,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        Ok(Locator::instance()
            .non_secret_controller
            .update_record_value(
                self.wallet_handle,
                category.into(),
                name.into(),
                new_value.into(),
            )
            .await?)
    }

    async fn delete_record(&self, category: &str, name: &str) -> VcxCoreResult<()> {
        Ok(Locator::instance()
            .non_secret_controller
            .delete_record(self.wallet_handle, category.into(), name.into())
            .await?)
    }

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        let json_filter = search_filter
            .map(|filter| match filter {
                SearchFilter::JsonFilter(inner) => Ok::<String, AriesVcxCoreError>(inner),
            })
            .transpose()?;

        let query_json = json_filter.unwrap_or("{}".into());

        let search_handle = Locator::instance()
            .non_secret_controller
            .open_search(
                self.wallet_handle,
                category.into(),
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
        while let Some(record) = next().await? {
            records.push(record.into());
        }

        Ok(records)
    }
}
