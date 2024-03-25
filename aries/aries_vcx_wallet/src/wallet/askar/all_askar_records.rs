use async_trait::async_trait;

use crate::{
    errors::error::VcxWalletResult,
    wallet::base_wallet::record::{AllRecords, PartialRecord},
};

pub struct AllAskarRecords {
    iterator: std::vec::IntoIter<PartialRecord>,
    total_count: Option<usize>,
}

impl AllAskarRecords {
    pub fn new(iterator: std::vec::IntoIter<PartialRecord>, total_count: Option<usize>) -> Self {
        Self {
            iterator,
            total_count,
        }
    }
}

#[async_trait]
impl AllRecords for AllAskarRecords {
    fn total_count(&self) -> VcxWalletResult<Option<usize>> {
        Ok(self.total_count)
    }

    async fn next(&mut self) -> VcxWalletResult<Option<PartialRecord>> {
        Ok(self.iterator.next())
    }
}
