use async_trait::async_trait;

use super::{
    record::{AllRecords, Record},
    record_category::RecordCategory,
    search_filter::SearchFilter,
};
use crate::{errors::error::VcxCoreResult, wallet::record_tags::RecordTags};

#[async_trait]
pub trait RecordWallet {
    async fn all_records(&self) -> VcxCoreResult<Box<dyn AllRecords + Send>>;

    async fn add_record(&self, record: Record) -> VcxCoreResult<()>;

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<Record>;

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxCoreResult<()>;

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()>;

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<()>;

    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>>;
}
