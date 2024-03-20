use async_trait::async_trait;

use super::{
    record::{AllRecords, Record},
    record_category::RecordCategory,
    // search_filter::SearchFilter,
};
use crate::{errors::error::VcxWalletResult, wallet::record_tags::RecordTags};

#[async_trait]
pub trait RecordWallet {
    async fn all_records(&self) -> VcxWalletResult<Box<dyn AllRecords + Send>>;

    async fn add_record(&self, record: Record) -> VcxWalletResult<()>;

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<Record>;

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxWalletResult<()>;

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxWalletResult<()>;

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<()>;

    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<String>,
    ) -> VcxWalletResult<Vec<Record>>;
}
