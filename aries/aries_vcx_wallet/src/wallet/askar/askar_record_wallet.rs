use std::str::FromStr;

use aries_askar::entry::{EntryTag, TagFilter};
use async_trait::async_trait;

use super::{all_askar_records::AllAskarRecords, AskarWallet};
use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{
        base_wallet::{
            record::{AllRecords, PartialRecord, Record},
            record_category::RecordCategory,
            record_wallet::RecordWallet,
        },
        record_tags::RecordTags,
    },
};

#[async_trait]
impl RecordWallet for AskarWallet {
    async fn add_record(&self, record: Record) -> VcxWalletResult<()> {
        let tags: Option<Vec<EntryTag>> = Some(record.tags().clone().into());
        Ok(self
            .session()
            .await?
            .insert(
                &record.category().to_string(),
                record.name(),
                record.value().as_bytes(),
                tags.as_deref(),
                None,
            )
            .await?)
    }

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<Record> {
        let mut session = self.session().await?;

        Ok(self
            .fetch(&mut session, category, name, false)
            .await
            .map(TryFrom::try_from)??)
    }

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxWalletResult<()> {
        let mut session = self.session().await?;
        let askar_tags: Vec<EntryTag> = new_tags.into();
        let entry = self.fetch(&mut session, category, name, true).await?;

        Ok(session
            .replace(
                &category.to_string(),
                name,
                &entry.value,
                Some(&askar_tags),
                None,
            )
            .await?)
    }

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxWalletResult<()> {
        let mut session = self.session().await?;
        let entry = self.fetch(&mut session, category, name, true).await?;

        Ok(session
            .replace(
                &category.to_string(),
                name,
                new_value.as_bytes(),
                Some(&entry.tags),
                None,
            )
            .await?)
    }

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<()> {
        Ok(self
            .session()
            .await?
            .remove(&category.to_string(), name)
            .await?)
    }

    #[allow(unreachable_patterns)]
    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<String>,
    ) -> VcxWalletResult<Vec<Record>> {
        let filter = search_filter
            .map(|inner| TagFilter::from_str(&inner))
            .transpose()
            .map_err(|err| VcxWalletError::InvalidInput(err.to_string()))?;

        Ok(self
            .session()
            .await?
            .fetch_all(Some(&category.to_string()), filter, None, None, true, false)
            .await?
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Vec<Result<Record, _>>>()
            .into_iter()
            .collect::<Result<_, _>>()?)
    }

    async fn all_records(&self) -> VcxWalletResult<Box<dyn AllRecords + Send>> {
        let mut session = self.session().await?;

        let recs = session
            .fetch_all(None, None, None, None, true, false)
            .await?;

        let mut recs = recs
            .into_iter()
            .map(PartialRecord::from_askar_entry)
            .collect::<Result<Vec<_>, _>>()?;

        let keys = session
            .fetch_all_keys(None, None, None, None, false)
            .await?;

        let mut local_keys = keys
            .into_iter()
            .map(PartialRecord::from_askar_key_entry)
            .collect::<Result<Vec<_>, _>>()?;

        recs.append(&mut local_keys);

        let total_count = recs.len();

        Ok(Box::new(AllAskarRecords::new(
            recs.into_iter(),
            Some(total_count),
        )))
    }
}
