use async_trait::async_trait;
use indy_api_types::domain::wallet::IndyRecord;
use vdrtools::Locator;

use super::{all_indy_records::AllIndyRecords, indy_tags::IndyTags, WALLET_OPTIONS};
use crate::{
    errors::error::VcxCoreResult,
    wallet::{
        base_wallet::{
            record::{AllRecords, Record},
            record_category::RecordCategory,
            record_wallet::RecordWallet,
            search_filter::SearchFilter,
        },
        indy::IndySdkWallet,
        record_tags::RecordTags,
    },
};

#[async_trait]
impl RecordWallet for IndySdkWallet {
    async fn all_records(&self) -> VcxCoreResult<Box<dyn AllRecords + Send>> {
        let all = Locator::instance()
            .wallet_controller
            .get_all(self.get_wallet_handle())
            .await?;

        Ok(Box::new(AllIndyRecords::new(all)))
    }

    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        let tags_map = if record.tags().is_empty() {
            None
        } else {
            Some(IndyTags::from_record_tags(record.tags().clone()).into_inner())
        };

        Ok(Locator::instance()
            .non_secret_controller
            .add_record(
                self.wallet_handle,
                record.category().to_string(),
                record.name().into(),
                record.value().into(),
                tags_map,
            )
            .await?)
    }

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<Record> {
        let res = Locator::instance()
            .non_secret_controller
            .get_record(
                self.wallet_handle,
                category.to_string(),
                name.into(),
                WALLET_OPTIONS.into(),
            )
            .await?;

        let indy_record: IndyRecord = serde_json::from_str(&res)?;

        Ok(Record::try_from_indy_record(indy_record)?)
    }

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxCoreResult<()> {
        Ok(Locator::instance()
            .non_secret_controller
            .update_record_tags(
                self.wallet_handle,
                category.to_string(),
                name.into(),
                IndyTags::from_record_tags(new_tags).into_inner(),
            )
            .await?)
    }

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        Ok(Locator::instance()
            .non_secret_controller
            .update_record_value(
                self.wallet_handle,
                category.to_string(),
                name.into(),
                new_value.into(),
            )
            .await?)
    }

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<()> {
        Ok(Locator::instance()
            .non_secret_controller
            .delete_record(self.wallet_handle, category.to_string(), name.into())
            .await?)
    }

    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        self.search(category, search_filter).await
    }
}
