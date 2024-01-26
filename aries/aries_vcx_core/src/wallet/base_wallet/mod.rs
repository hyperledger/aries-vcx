use async_trait::async_trait;
use public_key::Key;

use super::entry_tag::EntryTags;
use crate::{
    errors::error::VcxCoreResult,
    wallet::{
        base_wallet::{did_data::DidData, record::Record, search_filter::SearchFilter},
        structs_io::UnpackMessageOutput,
    },
};

pub mod did_data;
pub mod record;
pub mod search_filter;

pub trait BaseWallet: RecordWallet + DidWallet + Send + Sync + std::fmt::Debug {}

#[async_trait]
pub trait DidWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        kdf_method_name: Option<&str>,
    ) -> VcxCoreResult<DidData>;

    async fn key_for_did(&self, did: &str) -> VcxCoreResult<Key>;

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxCoreResult<Key>;

    async fn replace_did_key_apply(&self, did: &str) -> VcxCoreResult<()>;

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxCoreResult<Vec<u8>>;

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool>;

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>>;

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput>;
}

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

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, Rng};

    use super::BaseWallet;
    use crate::{
        errors::error::AriesVcxCoreErrorKind,
        wallet::{
            base_wallet::{DidWallet, Record, RecordWallet},
            entry_tag::{EntryTag, EntryTags},
        },
    };

    fn random_seed() -> String {
        rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    async fn build_test_wallet() -> impl BaseWallet {
        #[cfg(feature = "vdrtools_wallet")]
        {
            use crate::wallet::indy::test::dev_setup_indy_wallet;
            dev_setup_indy_wallet().await
        }
    }

    #[tokio::test]
    async fn did_wallet_should_sign_and_verify() {
        let wallet = build_test_wallet().await;

        let did_data = wallet
            .create_and_store_my_did(Some(&random_seed()), None)
            .await
            .unwrap();

        let msg = "sign this".as_bytes();
        let sig = wallet.sign(did_data.verkey(), msg).await.unwrap();

        let res = wallet.verify(did_data.verkey(), msg, &sig).await.unwrap();
        assert!(res);
    }

    #[tokio::test]
    async fn did_wallet_should_rotate_keys() {
        let wallet = build_test_wallet().await;

        let did_data = wallet
            .create_and_store_my_did(Some(&random_seed()), None)
            .await
            .unwrap();

        let key = wallet.key_for_did(did_data.did()).await.unwrap();

        assert_eq!(did_data.verkey(), &key);

        let res = wallet
            .replace_did_key_start(did_data.did(), Some(&random_seed()))
            .await
            .unwrap();

        wallet.replace_did_key_apply(did_data.did()).await.unwrap();

        let new_key = wallet.key_for_did(did_data.did()).await.unwrap();
        assert_eq!(res, new_key);
    }

    #[tokio::test]
    async fn did_wallet_should_pack_and_unpack() {
        let wallet = build_test_wallet().await;

        let sender_data = wallet.create_and_store_my_did(None, None).await.unwrap();

        let receiver_data = wallet.create_and_store_my_did(None, None).await.unwrap();

        let msg = "pack me";

        let packed = wallet
            .pack_message(
                Some(sender_data.verkey().clone()),
                vec![receiver_data.verkey().clone()],
                msg.as_bytes(),
            )
            .await
            .unwrap();

        let unpacked = wallet.unpack_message(&packed).await.unwrap();

        assert_eq!(msg, unpacked.message);
    }

    #[tokio::test]
    async fn record_wallet_should_create_record() {
        let wallet = build_test_wallet().await;

        let name = "foo";
        let category = "my";
        let value = "bar";

        let record1 = Record::builder()
            .name(name.into())
            .category(category.into())
            .value(value.into())
            .build();
        let record2 = Record::builder()
            .name("baz".into())
            .category(category.into())
            .value("box".into())
            .build();

        wallet.add_record(record1).await.unwrap();
        wallet.add_record(record2).await.unwrap();

        let res = wallet.get_record(category, name).await.unwrap();

        assert_eq!(value, res.value());
    }

    #[tokio::test]
    async fn record_wallet_should_delete_record() {
        let wallet = build_test_wallet().await;

        let name = "foo";
        let category = "my";
        let value = "bar";

        let record = Record::builder()
            .name(name.into())
            .category(category.into())
            .value(value.into())
            .build();

        wallet.add_record(record).await.unwrap();

        let res = wallet.get_record(category, name).await.unwrap();

        assert_eq!(value, res.value());

        wallet.delete_record(category, name).await.unwrap();

        let err = wallet.get_record(category, name).await.unwrap_err();
        assert_eq!(AriesVcxCoreErrorKind::WalletRecordNotFound, err.kind());
    }

    #[tokio::test]
    async fn record_wallet_should_search_for_records() {
        let wallet = build_test_wallet().await;

        let name1 = "foo";
        let name2 = "foa";
        let name3 = "fob";
        let category1 = "my";
        let category2 = "your";
        let value = "xxx";

        let record1 = Record::builder()
            .name(name1.into())
            .category(category1.into())
            .value(value.into())
            .build();
        wallet.add_record(record1).await.unwrap();

        let record2 = Record::builder()
            .name(name2.into())
            .category(category1.into())
            .value(value.into())
            .build();
        wallet.add_record(record2).await.unwrap();

        let record3 = Record::builder()
            .name(name3.into())
            .category(category2.into())
            .value(value.into())
            .build();
        wallet.add_record(record3).await.unwrap();

        let res = wallet.search_record(category1, None).await.unwrap();

        assert_eq!(2, res.len());
    }

    #[tokio::test]
    async fn record_wallet_should_update_record() {
        let wallet = build_test_wallet().await;

        let name = "foo";
        let category = "my";
        let value1 = "xxx";
        let value2 = "yyy";
        let tags1: EntryTags = vec![EntryTag::Plaintext("a".into(), "b".into())].into();
        let tags2 = EntryTags::default();

        let record = Record::builder()
            .name(name.into())
            .category(category.into())
            .tags(tags1.clone())
            .value(value1.into())
            .build();
        wallet.add_record(record.clone()).await.unwrap();

        wallet
            .update_record_value(category, name, value2)
            .await
            .unwrap();
        wallet
            .update_record_tags(category, name, tags2.clone())
            .await
            .unwrap();

        let res = wallet.get_record(category, name).await.unwrap();
        assert_eq!(value2, res.value());
        assert_eq!(&tags2, res.tags());
    }

    #[tokio::test]
    async fn record_wallet_should_update_only_value() {
        let wallet = build_test_wallet().await;

        let name = "foo";
        let category = "my";
        let value1 = "xxx";
        let value2 = "yyy";
        let tags: EntryTags = vec![EntryTag::Plaintext("a".into(), "b".into())].into();

        let record = Record::builder()
            .name(name.into())
            .category(category.into())
            .tags(tags.clone())
            .value(value1.into())
            .build();
        wallet.add_record(record.clone()).await.unwrap();

        wallet
            .update_record_value(category, name, value2)
            .await
            .unwrap();

        let res = wallet.get_record(category, name).await.unwrap();
        assert_eq!(value2, res.value());
        assert_eq!(&tags, res.tags());
    }

    #[tokio::test]
    async fn record_wallet_should_update_only_tags() {
        let wallet = build_test_wallet().await;

        let name = "foo";
        let category = "my";
        let value = "xxx";
        let tags1: EntryTags = vec![EntryTag::Plaintext("a".into(), "b".into())].into();
        let tags2: EntryTags = vec![EntryTag::Plaintext("c".into(), "d".into())].into();

        let record = Record::builder()
            .name(name.into())
            .category(category.into())
            .tags(tags1.clone())
            .value(value.into())
            .build();
        wallet.add_record(record.clone()).await.unwrap();

        wallet
            .update_record_tags(category, name, tags2.clone())
            .await
            .unwrap();

        let res = wallet.get_record(category, name).await.unwrap();
        assert_eq!(value, res.value());
        assert_eq!(&tags2, res.tags());
    }
}
