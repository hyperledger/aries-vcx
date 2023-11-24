use aries_askar::entry::{Entry, EntryTag, TagFilter};

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind},
    wallet2::askar_wallet::SecretBytes,
};
use async_trait::async_trait;

use crate::{errors::error::VcxCoreResult, wallet2::RecordWallet};

use super::AskarWallet;

#[derive(Default, Clone)]
pub struct Record {
    pub category: String,
    pub name: String,
    pub value: SecretBytes,
    pub tags: Option<Vec<EntryTag>>,
    pub expiry_ms: Option<i64>,
}

impl Record {
    pub fn set_name(mut self, new_name: &str) -> Self {
        self.name = new_name.to_owned();
        self
    }

    pub fn set_category(mut self, new_category: &str) -> Self {
        self.category = new_category.to_owned();
        self
    }

    pub fn set_value(mut self, new_value: &SecretBytes) -> Self {
        self.value = new_value.to_owned();
        self
    }

    pub fn set_tags(mut self, new_tags: Vec<EntryTag>) -> Self {
        self.tags = Some(new_tags);
        self
    }

    pub fn set_expiry_ms(mut self, new_expiry_ms: i64) -> Self {
        self.expiry_ms = Some(new_expiry_ms);
        self
    }
}

#[derive(Default)]
pub struct RecordId {
    name: String,
    category: String,
    for_update: bool,
}

impl RecordId {
    pub fn set_name(mut self, new_name: &str) -> Self {
        self.name = new_name.to_string();
        self
    }

    pub fn set_category(mut self, new_category: &str) -> Self {
        self.category = new_category.to_string();
        self
    }

    pub fn set_for_update(mut self, new_for_update: bool) -> Self {
        self.for_update = new_for_update;
        self
    }
}

#[derive(Default)]
pub struct SearchFilter {
    category: Option<String>,
    tag_filter: Option<TagFilter>,
    limit: Option<i64>,
}

impl SearchFilter {
    pub fn set_category(mut self, new_category: &str) -> Self {
        self.category = Some(new_category.to_string());
        self
    }

    pub fn set_tag_filter(mut self, new_tag_filter: TagFilter) -> Self {
        self.tag_filter = Some(new_tag_filter);
        self
    }

    pub fn set_limit(mut self, new_limit: i64) -> Self {
        self.limit = Some(new_limit);
        self
    }
}

#[async_trait]
impl RecordWallet for AskarWallet {
    type Record = Record;
    type RecordId = RecordId;
    type FoundRecord = Entry;
    type SearchFilter = SearchFilter;

    async fn add_record(&self, record: Self::Record) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        Ok(session
            .insert(
                &record.category,
                &record.name,
                &record.value,
                record.tags.as_deref(),
                record.expiry_ms,
            )
            .await?)
    }

    async fn get_record(&self, id: &Self::RecordId) -> VcxCoreResult<Self::FoundRecord> {
        let mut session = self.backend.session(self.profile.clone()).await?;

        session
            .fetch(&id.category, &id.name, id.for_update)
            .await?
            .ok_or_else(|| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::WalletRecordNotFound,
                    "record not found",
                )
            })
    }

    async fn update_record(&self, record: Self::Record) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        Ok(session
            .replace(
                &record.category,
                &record.name,
                &record.value,
                record.tags.as_deref(),
                record.expiry_ms,
            )
            .await?)
    }

    async fn delete_record(&self, id: &Self::RecordId) -> VcxCoreResult<()> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        Ok(session.remove(&id.category, &id.name).await?)
    }

    async fn search_record(
        &self,
        filter: Self::SearchFilter,
    ) -> VcxCoreResult<Vec<Self::FoundRecord>> {
        let mut session = self.backend.session(self.profile.clone()).await?;
        Ok(session
            .fetch_all(
                filter.category.as_deref(),
                filter.tag_filter,
                filter.limit,
                false,
            )
            .await?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aries_askar::StoreKeyMethod;
    use uuid::Uuid;

    use crate::wallet2::askar_wallet::AskarWallet;

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

        let record1 = Record::default()
            .set_name(&name1)
            .set_category(&category1)
            .set_value(&value1.into());

        wallet.add_record(record1).await.unwrap();

        let name2 = "do-not-delete-me".to_string();
        let category2 = "my".to_string();
        let value2 = "gg".to_string();

        let record2 = Record::default()
            .set_name(&name2)
            .set_category(&category2)
            .set_value(&value2.into());

        wallet.add_record(record2).await.unwrap();

        let record1_id = RecordId::default()
            .set_name(&name1)
            .set_category(&category1);
        wallet.delete_record(&record1_id).await.unwrap();
        let err = wallet.get_record(&record1_id).await.unwrap_err();
        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err.kind());

        let record2_id = RecordId::default()
            .set_name(&name2)
            .set_category(&category2);
        wallet.get_record(&record2_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_askar_should_get_record() {
        let wallet = create_test_wallet().await;

        let name1 = "foobar".to_string();
        let category1 = "my".to_string();
        let value1 = "ff".to_string();

        let record1 = Record::default()
            .set_name(&name1)
            .set_category(&category1)
            .set_value(&value1.clone().into());

        wallet.add_record(record1).await.unwrap();

        let name2 = "foofar".to_string();
        let category2 = "your".to_string();
        let value2 = "gg".to_string();

        let record2 = Record::default()
            .set_name(&name2)
            .set_category(&category2)
            .set_value(&value2.clone().into());

        wallet.add_record(record2).await.unwrap();

        let record1_id = RecordId::default()
            .set_name(&name1)
            .set_category(&category1);
        let found1 = wallet.get_record(&record1_id).await.unwrap();
        assert_eq!(value1, secret_bytes_to_string(&found1.value));

        let record3_id = RecordId::default()
            .set_name(&name1)
            .set_category(&category2);
        let err1 = wallet.get_record(&record3_id).await.unwrap_err();

        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err1.kind())
    }

    #[tokio::test]
    async fn test_askar_should_update_record() {
        let wallet = create_test_wallet().await;

        let name = "test-name".to_string();
        let category = "test-category".to_string();
        let value = "test-value".to_string();

        let record = Record::default()
            .set_name(&name)
            .set_category(&category)
            .set_value(&value.clone().into());

        wallet.add_record(record.clone()).await.unwrap();

        let updated_value = "updated-test-value".to_string();
        let record = record.set_value(&updated_value.clone().into());

        wallet.update_record(record.clone()).await.unwrap();

        let record_id = RecordId::default().set_name(&name).set_category(&category);
        let found = wallet.get_record(&record_id).await.unwrap();
        assert_eq!(updated_value, secret_bytes_to_string(&found.value));

        let other_category = "other-test-category".to_string();
        let record = record.set_category(&other_category);
        let err = wallet.update_record(record.clone()).await.unwrap_err();

        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err.kind());
    }

    fn secret_bytes_to_string(sb: &SecretBytes) -> String {
        std::str::from_utf8(&sb.to_vec()).unwrap().to_string()
    }

    #[tokio::test]
    async fn test_askar_should_find_records() {
        let wallet = create_test_wallet().await;

        let category = "my".to_string();

        let record1 = Record::default()
            .set_name("first record".into())
            .set_category(&category);
        wallet.add_record(record1).await.unwrap();

        let record2 = Record::default()
            .set_name("second record")
            .set_category(&category);
        wallet.add_record(record2).await.unwrap();

        let record3 = Record::default()
            .set_name("third record")
            .set_category("your");
        wallet.add_record(record3).await.unwrap();

        let filter = SearchFilter::default().set_category(&category);

        let all = wallet.search_record(filter).await.unwrap();

        assert_eq!(2, all.len());
    }
}
