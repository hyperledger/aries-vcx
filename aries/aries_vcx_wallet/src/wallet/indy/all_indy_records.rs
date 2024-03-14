use async_trait::async_trait;
use vdrtools::indy_wallet::iterator::WalletIterator;

use crate::{
    errors::error::VcxWalletResult,
    wallet::base_wallet::record::{AllRecords, PartialRecord},
};

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
    fn total_count(&self) -> VcxWalletResult<Option<usize>> {
        Ok(self.iterator.get_total_count()?)
    }

    async fn next(&mut self) -> VcxWalletResult<Option<PartialRecord>> {
        let item = self.iterator.next().await?;

        Ok(item.map(PartialRecord::from_wallet_record))
    }
}
