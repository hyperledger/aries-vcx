use std::sync::Arc;

use async_trait::async_trait;

use crate::{errors::error::VcxCoreResult, wallet::entry_tag::EntryTags};

use super::{record::Record, search_filter::SearchFilter, BaseWallet};

#[async_trait]
pub trait RecordWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()>;

    async fn get_record(&self, category: &str, name: &str) -> VcxCoreResult<Record>;

    async fn update_record_tags(
        &self,
        category: &str,
        name: &str,
        new_tags: EntryTags,
    ) -> VcxCoreResult<()>;

    async fn update_record_value(
        &self,
        category: &str,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()>;

    async fn delete_record(&self, category: &str, name: &str) -> VcxCoreResult<()>;

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>>;
}

#[async_trait]
impl RecordWallet for Arc<dyn BaseWallet> {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        self.as_ref().add_record(record).await
    }

    async fn get_record(&self, category: &str, name: &str) -> VcxCoreResult<Record> {
        self.as_ref().get_record(category, name).await
    }

    async fn update_record_tags(
        &self,
        category: &str,
        name: &str,
        new_tags: EntryTags,
    ) -> VcxCoreResult<()> {
        self.as_ref()
            .update_record_tags(category, name, new_tags)
            .await
    }

    async fn update_record_value(
        &self,
        category: &str,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        self.as_ref()
            .update_record_value(category, name, new_value)
            .await
    }

    async fn delete_record(&self, category: &str, name: &str) -> VcxCoreResult<()> {
        self.as_ref().delete_record(category, name).await
    }

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        self.as_ref().search_record(category, search_filter).await
    }
}
