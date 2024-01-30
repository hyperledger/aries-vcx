use vdrtools::WalletRecord;

use super::indy_tags::IndyTags;
use crate::wallet::base_wallet::record::PartialRecord;

impl PartialRecord {
    pub fn from_wallet_record(wallet_record: WalletRecord) -> Self {
        let name = wallet_record.get_id().into();
        let category = wallet_record.get_type();
        let value = wallet_record.get_value();

        let found_tags = wallet_record.get_tags();

        Self::builder()
            .name(name)
            .category(category.map(Into::into))
            .value(value.map(Into::into))
            .tags(found_tags.map(|tags| IndyTags::new(tags.clone()).into_record_tags()))
            .build()
    }
}
