use aries_askar::entry::EntryTag;
use async_trait::async_trait;

use super::AskarWallet;
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::{
        base_wallet::{
            record::Record, record_category::RecordCategory, search_filter::SearchFilter,
            RecordWallet,
        },
        record_tags::RecordTags,
    },
};

#[async_trait]
impl RecordWallet for AskarWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
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

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<Record> {
        Ok(self
            .session()
            .await?
            .fetch(&category.to_string(), name, false)
            .await?
            .ok_or_else(|| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletRecordNotFound,
                    "record not found",
                )
            })
            .map(TryFrom::try_from)??)
    }

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxCoreResult<()> {
        let mut session = self.session().await?;
        let askar_tags: Vec<EntryTag> = new_tags.into();
        match session.fetch(&category.to_string(), name, true).await? {
            Some(record) => Ok(session
                .replace(
                    &category.to_string(),
                    name,
                    &record.value,
                    Some(&askar_tags),
                    None,
                )
                .await?),
            None => Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::WalletRecordNotFound,
                "wallet record not found",
            )),
        }
    }

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        let mut session = self.session().await?;
        match session.fetch(&category.to_string(), name, true).await? {
            Some(record) => Ok(session
                .replace(
                    &category.to_string(),
                    name,
                    new_value.as_bytes(),
                    Some(&record.tags),
                    None,
                )
                .await?),
            None => Err(AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::WalletRecordNotFound,
                "wallet record not found",
            )),
        }
    }

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<()> {
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
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        Ok(self
            .session()
            .await?
            .fetch_all(
                Some(&category.to_string()),
                search_filter
                    .map(|filter| match filter {
                        SearchFilter::TagFilter(inner) => Ok(inner),
                        _ => Err(AriesVcxCoreError::from_msg(
                            AriesVcxCoreErrorKind::WalletError,
                            "unsupported search filter",
                        )),
                    })
                    .transpose()?,
                None,
                false,
            )
            .await?
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Vec<Result<Record, _>>>()
            .into_iter()
            .collect::<Result<_, _>>()?)
    }
}
