use aries_askar::entry::{Entry, EntryTag as AskarEntryTag};
use async_trait::async_trait;

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet2::{Record, RecordWallet, SearchFilter},
};

use super::AskarWallet;

#[async_trait]
impl RecordWallet for AskarWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        let tags: Option<Vec<AskarEntryTag>> =
            Some(record.tags.into_iter().map(From::from).collect());

        Ok(session
            .insert(
                &record.category,
                &record.name,
                &record.value.as_bytes(),
                tags.as_deref(),
                None,
            )
            .await?)
    }

    async fn get_record(&self, name: &str, category: &str) -> VcxCoreResult<Record> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        let res = session
            .fetch(category, &name, false)
            .await?
            .ok_or_else(|| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletRecordNotFound,
                    "record not found",
                )
            })
            .map(TryFrom::try_from)??;

        Ok(res)
    }

    async fn update_record(&self, record: Record) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        let entry: Entry = record.into();

        Ok(session
            .replace(
                &entry.category,
                &entry.name,
                &entry.value,
                Some(&entry.tags),
                None,
            )
            .await?)
    }

    async fn delete_record(&self, name: &str, category: &str) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        Ok(session.remove(&category, &name).await?)
    }

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        let tag_filter = search_filter
            .map(|filter| match filter {
                SearchFilter::TagFilter(inner) => Ok(inner),
                _ => Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletUnexpected,
                    "unsupported search filter",
                )),
            })
            .transpose()?;

        let mut session = self.backend.session(self.profile.clone()).await?;
        let res = session
            .fetch_all(Some(category), tag_filter, None, false)
            .await?;

        let rs: Vec<_> = res
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Vec<Result<Record, _>>>()
            .into_iter()
            .collect::<Result<_, _>>()?;
        Ok(rs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aries_askar::StoreKeyMethod;
    use uuid::Uuid;

    use crate::wallet2::{askar_wallet::AskarWallet, RecordBuilder};

    async fn create_test_wallet() -> AskarWallet {
        AskarWallet::create(
            "sqlite://:memory:",
            StoreKeyMethod::Unprotected,
            None.into(),
            true,
            Some(Uuid::new_v4().to_string()),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_askar_should_delete_record() {
        let wallet = create_test_wallet().await;

        let name1 = "delete-me-1".to_string();
        let category1 = "my".to_string();
        let value1 = "ff".to_string();

        let record1 = RecordBuilder::default()
            .name(name1.clone())
            .category(category1.clone())
            .value(value1.clone().into())
            .build()
            .unwrap();

        wallet.add_record(record1).await.unwrap();

        let name2 = "do-not-delete-me".to_string();
        let category2 = "my".to_string();
        let value2 = "gg".to_string();

        let record2 = RecordBuilder::default()
            .name(name2.clone())
            .category(category2.clone())
            .value(value2.clone().into())
            .build()
            .unwrap();

        wallet.add_record(record2).await.unwrap();

        wallet.delete_record(&name1, &category1).await.unwrap();
        let err = wallet.get_record(&name1, &category1).await.unwrap_err();
        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err.kind());

        wallet.get_record(&name2, &category2).await.unwrap();
    }

    #[tokio::test]
    async fn test_askar_should_get_record() {
        let wallet = create_test_wallet().await;

        let name1 = "foobar".to_string();
        let category1 = "my".to_string();
        let value1 = "ff".to_string();

        let record1 = RecordBuilder::default()
            .name(name1.clone())
            .category(category1.clone())
            .value(value1.clone().into())
            .build()
            .unwrap();

        wallet.add_record(record1).await.unwrap();

        let name2 = "foofar".to_string();
        let category2 = "your".to_string();
        let value2 = "gg".to_string();

        let record2 = RecordBuilder::default()
            .name(name2.clone())
            .category(category2.clone())
            .value(value2.clone().into())
            .build()
            .unwrap();

        wallet.add_record(record2).await.unwrap();

        let found1 = wallet.get_record(&name1, &category1).await.unwrap();
        assert_eq!(value1, found1.value);

        let err1 = wallet.get_record(&name1, &category2).await.unwrap_err();

        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err1.kind())
    }

    #[tokio::test]
    async fn test_askar_should_update_record() {
        let wallet = create_test_wallet().await;

        let name = "test-name".to_string();
        let category = "test-category".to_string();
        let value = "test-value".to_string();

        let record = RecordBuilder::default()
            .name(name.clone())
            .category(category.clone())
            .value(value.clone().into())
            .build()
            .unwrap();

        wallet.add_record(record.clone()).await.unwrap();

        let updated_value = "updated-test-value".to_string();

        let record = RecordBuilder::default()
            .name(name.clone())
            .category(category.clone())
            .value(updated_value.clone().into())
            .build()
            .unwrap();

        wallet.update_record(record.clone()).await.unwrap();

        let found = wallet.get_record(&name, &category).await.unwrap();
        assert_eq!(updated_value, found.value);

        let other_category = "other-test-category".to_string();
        let record = RecordBuilder::default()
            .name(name.clone())
            .category(other_category.clone())
            .value(updated_value.clone().into())
            .build()
            .unwrap();

        let err = wallet.update_record(record.clone()).await.unwrap_err();

        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err.kind());
    }

    #[tokio::test]
    async fn test_askar_should_find_records() {
        let wallet = create_test_wallet().await;

        let category = "my".to_string();

        let record1 = RecordBuilder::default()
            .name("first record".into())
            .category(category.clone())
            .build()
            .unwrap();
        wallet.add_record(record1).await.unwrap();

        let record2 = RecordBuilder::default()
            .name("second record".into())
            .category(category.clone())
            .build()
            .unwrap();
        wallet.add_record(record2).await.unwrap();

        let record3 = RecordBuilder::default()
            .name("third record".into())
            .category("your".into())
            .build()
            .unwrap();
        wallet.add_record(record3).await.unwrap();

        let all = wallet.search_record(&category, None).await.unwrap();

        assert_eq!(2, all.len());
    }
}
