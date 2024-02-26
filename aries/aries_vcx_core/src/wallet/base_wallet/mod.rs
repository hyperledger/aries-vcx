use async_trait::async_trait;
use public_key::Key;

use self::{did_data::DidData, record_category::RecordCategory};
use super::record_tags::RecordTags;
use crate::{
    errors::error::VcxCoreResult,
    wallet::{
        base_wallet::{record::Record, search_filter::SearchFilter},
        structs_io::UnpackMessageOutput,
    },
};

pub mod did_data;
pub mod did_value;
pub mod record;
pub mod record_category;
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

    async fn key_count(&self) -> VcxCoreResult<usize>;

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

#[cfg(test)]
mod tests {
    use super::BaseWallet;
    use crate::{
        errors::error::AriesVcxCoreErrorKind,
        wallet::{
            base_wallet::{record_category::RecordCategory, Record},
            record_tags::{RecordTag, RecordTags},
            utils::{did_from_key, random_seed},
        },
    };

    #[allow(unused_variables)]
    async fn build_test_wallet() -> Box<dyn BaseWallet> {
        #[cfg(feature = "vdrtools_wallet")]
        let wallet = {
            use crate::wallet::indy::tests::dev_setup_indy_wallet;
            dev_setup_indy_wallet().await
        };

        #[cfg(feature = "askar_wallet")]
        let wallet = {
            use crate::wallet::askar::tests::dev_setup_askar_wallet;
            dev_setup_askar_wallet().await
        };

        wallet
    }

    #[tokio::test]
    async fn did_wallet_should_create_and_store_did_atomically() {
        let wallet = build_test_wallet().await;
        let seed = random_seed();
        wallet
            .create_and_store_my_did(Some(&seed), None)
            .await
            .unwrap();
        let _ = wallet.create_and_store_my_did(Some(&seed), None).await;
        let res = wallet.key_count().await.unwrap();

        assert_eq!(1, res)
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
    async fn did_wallet_should_return_correct_key() {
        let wallet = build_test_wallet().await;

        let first_data = wallet.create_and_store_my_did(None, None).await.unwrap();

        let new_key = wallet
            .replace_did_key_start(first_data.did(), Some(&random_seed()))
            .await
            .unwrap();

        assert_eq!(new_key.key().len(), 32);

        wallet
            .replace_did_key_apply(first_data.did())
            .await
            .unwrap();

        let new_verkey = wallet.key_for_did(first_data.did()).await.unwrap();
        assert_eq!(new_verkey.key().len(), 32);

        assert_eq!(
            did_from_key(new_key.clone()),
            did_from_key(new_verkey.clone())
        );
        assert_eq!(new_key.key(), new_verkey.key());
    }

    #[tokio::test]
    async fn did_wallet_should_replace_did_key_repeatedly() {
        let wallet = build_test_wallet().await;

        let first_data = wallet.create_and_store_my_did(None, None).await.unwrap();

        let new_key = wallet
            .replace_did_key_start(first_data.did(), Some(&random_seed()))
            .await
            .unwrap();

        wallet
            .replace_did_key_apply(first_data.did())
            .await
            .unwrap();

        let new_verkey = wallet.key_for_did(first_data.did()).await.unwrap();

        assert_eq!(did_from_key(new_key), did_from_key(new_verkey));

        let second_new_key = wallet
            .replace_did_key_start(first_data.did(), Some(&random_seed()))
            .await
            .unwrap();

        wallet
            .replace_did_key_apply(first_data.did())
            .await
            .unwrap();

        let second_new_verkey = wallet.key_for_did(first_data.did()).await.unwrap();

        assert_eq!(
            did_from_key(second_new_key),
            did_from_key(second_new_verkey)
        );
    }

    #[tokio::test]
    async fn did_wallet_should_replace_did_key_interleaved() {
        let wallet = build_test_wallet().await;

        let first_data = wallet.create_and_store_my_did(None, None).await.unwrap();

        let second_data = wallet
            .create_and_store_my_did(Some(&random_seed()), None)
            .await
            .unwrap();

        let first_new_key = wallet
            .replace_did_key_start(first_data.did(), Some(&random_seed()))
            .await
            .unwrap();

        let second_new_key = wallet
            .replace_did_key_start(second_data.did(), Some(&random_seed()))
            .await
            .unwrap();

        wallet
            .replace_did_key_apply(second_data.did())
            .await
            .unwrap();
        wallet
            .replace_did_key_apply(first_data.did())
            .await
            .unwrap();

        let first_new_verkey = wallet.key_for_did(first_data.did()).await.unwrap();
        let second_new_verkey = wallet.key_for_did(second_data.did()).await.unwrap();

        assert_eq!(did_from_key(first_new_key), did_from_key(first_new_verkey));
        assert_eq!(
            did_from_key(second_new_key),
            did_from_key(second_new_verkey)
        );
    }

    #[tokio::test]
    async fn did_wallet_should_pack_and_unpack_authcrypt() {
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
    async fn did_wallet_should_pack_and_unpack_anoncrypt() {
        let wallet = build_test_wallet().await;

        let receiver_data = wallet.create_and_store_my_did(None, None).await.unwrap();

        let msg = "pack me";

        let packed = wallet
            .pack_message(None, vec![receiver_data.verkey().clone()], msg.as_bytes())
            .await
            .unwrap();

        let unpacked = wallet.unpack_message(&packed).await.unwrap();

        assert_eq!(msg, unpacked.message);
    }

    #[tokio::test]
    async fn record_wallet_should_create_record() {
        let wallet = build_test_wallet().await;

        let name = "foo";
        let category = RecordCategory::default();
        let value = "bar";

        let record1 = Record::builder()
            .name(name.into())
            .category(category)
            .value(value.into())
            .build();
        let record2 = Record::builder()
            .name("baz".into())
            .category(category)
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
        let category = RecordCategory::default();
        let value = "bar";

        let record = Record::builder()
            .name(name.into())
            .category(category)
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
        let category1 = RecordCategory::Cred;
        let category2 = RecordCategory::default();
        let value = "xxx";

        let record1 = Record::builder()
            .name(name1.into())
            .category(category1)
            .value(value.into())
            .build();
        wallet.add_record(record1).await.unwrap();

        let record2 = Record::builder()
            .name(name2.into())
            .category(category1)
            .value(value.into())
            .build();
        wallet.add_record(record2).await.unwrap();

        let record3 = Record::builder()
            .name(name3.into())
            .category(category2)
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
        let category = RecordCategory::default();
        let value1 = "xxx";
        let value2 = "yyy";
        let tags1: RecordTags = vec![RecordTag::new("a", "b")].into();
        let tags2 = RecordTags::default();

        let record = Record::builder()
            .name(name.into())
            .category(category)
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
        let category = RecordCategory::default();
        let value1 = "xxx";
        let value2 = "yyy";
        let tags: RecordTags = vec![RecordTag::new("a", "b")].into();

        let record = Record::builder()
            .name(name.into())
            .category(category)
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
        let category = RecordCategory::default();
        let value = "xxx";
        let tags1: RecordTags = vec![RecordTag::new("a", "b")].into();
        let tags2: RecordTags = vec![RecordTag::new("c", "d")].into();

        let record = Record::builder()
            .name(name.into())
            .category(category)
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
