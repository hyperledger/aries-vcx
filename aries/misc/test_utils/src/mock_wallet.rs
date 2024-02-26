use aries_vcx_core::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::{
        base_wallet::{
            did_data::DidData, record::Record, record_category::RecordCategory,
            search_filter::SearchFilter, BaseWallet, DidWallet, RecordWallet,
        },
        record_tags::RecordTags,
        structs_io::UnpackMessageOutput,
    },
};
use async_trait::async_trait;
use public_key::{Key, KeyType};

#[derive(Debug)]
pub struct MockWallet;

impl BaseWallet for MockWallet {}

pub const DID: &str = "FhrSrYtQcw3p9xwf7NYemf";
pub const VERKEY: &str = "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE";

#[async_trait]
#[allow(unused_variables)]
impl RecordWallet for MockWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<Record> {
        Ok(Record::builder()
            .name("123".into())
            .category(RecordCategory::default())
            .value("record value".into())
            .build())
    }

    async fn update_record_value(
        &self,
        category: RecordCategory,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: search_record",
        ))
    }
}

#[async_trait]
#[allow(unused_variables)]
impl DidWallet for MockWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxCoreResult<DidData> {
        Ok(DidData::new(
            DID,
            &Key::new(VERKEY.into(), KeyType::Ed25519).unwrap(),
        ))
    }

    async fn key_count(&self) -> VcxCoreResult<usize> {
        Ok(0)
    }

    async fn key_for_did(&self, name: &str) -> VcxCoreResult<Key> {
        Ok(Key::new(VERKEY.into(), KeyType::Ed25519).unwrap())
    }

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxCoreResult<Key> {
        Ok(Key::new(VERKEY.into(), KeyType::Ed25519).unwrap())
    }

    async fn replace_did_key_apply(&self, did: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Ok(Vec::from(msg))
    }

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        Ok(true)
    }

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        Ok(Vec::from(msg))
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput> {
        Ok(UnpackMessageOutput {
            message: format!("{:?}", msg),
            recipient_verkey: "".to_owned(),
            sender_verkey: None,
        })
    }
}
