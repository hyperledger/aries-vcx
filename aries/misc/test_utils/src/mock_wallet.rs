use aries_vcx_wallet::{
    errors::error::{VcxWalletError, VcxWalletResult},
    wallet::{
        base_wallet::{
            did_data::DidData,
            did_wallet::DidWallet,
            issuer_config::IssuerConfig,
            key_value::KeyValue,
            record::{AllRecords, PartialRecord, Record},
            record_category::RecordCategory,
            record_wallet::RecordWallet,
            BaseWallet,
        },
        record_tags::RecordTags,
        structs_io::UnpackMessageOutput,
    },
};
use async_trait::async_trait;
use public_key::{Key, KeyType};

#[derive(Debug)]
pub struct MockWallet;

pub struct MockAllRecords;

#[async_trait]
impl AllRecords for MockAllRecords {
    fn total_count(&self) -> VcxWalletResult<Option<usize>> {
        Ok(Some(0))
    }

    async fn next(&mut self) -> VcxWalletResult<Option<PartialRecord>> {
        Ok(None)
    }
}

#[async_trait]
#[allow(unused_variables)]
impl BaseWallet for MockWallet {
    async fn export_wallet(&self, path: &str, backup_key: &str) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn close_wallet(&self) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn configure_issuer(&self, key_seed: &str) -> VcxWalletResult<IssuerConfig> {
        Ok(IssuerConfig::builder().build())
    }

    async fn create_key(
        &self,
        name: &str,
        value: KeyValue,
        tags: &RecordTags,
    ) -> VcxWalletResult<()> {
        Ok(())
    }
}

pub const DID: &str = "FhrSrYtQcw3p9xwf7NYemf";
pub const VERKEY: &str = "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE";

#[async_trait]
#[allow(unused_variables)]
impl RecordWallet for MockWallet {
    async fn all_records(&self) -> VcxWalletResult<Box<dyn AllRecords + Send>> {
        Ok(Box::new(MockAllRecords {}))
    }

    async fn add_record(&self, record: Record) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn get_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<Record> {
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
    ) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn update_record_tags(
        &self,
        category: RecordCategory,
        name: &str,
        new_tags: RecordTags,
    ) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn delete_record(&self, category: RecordCategory, name: &str) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn search_record(
        &self,
        category: RecordCategory,
        search_filter: Option<String>,
    ) -> VcxWalletResult<Vec<Record>> {
        Err(VcxWalletError::Unimplemented(
            "search_record is not implemented for MockWallet".into(),
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
    ) -> VcxWalletResult<DidData> {
        Ok(DidData::new(
            DID,
            &Key::new(VERKEY.into(), KeyType::Ed25519).unwrap(),
        ))
    }

    async fn key_count(&self) -> VcxWalletResult<usize> {
        Ok(0)
    }

    async fn key_for_did(&self, name: &str) -> VcxWalletResult<Key> {
        Ok(Key::new(VERKEY.into(), KeyType::Ed25519).unwrap())
    }

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxWalletResult<Key> {
        Ok(Key::new(VERKEY.into(), KeyType::Ed25519).unwrap())
    }

    async fn replace_did_key_apply(&self, did: &str) -> VcxWalletResult<()> {
        Ok(())
    }

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxWalletResult<Vec<u8>> {
        Ok(Vec::from(msg))
    }

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxWalletResult<bool> {
        Ok(true)
    }

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxWalletResult<Vec<u8>> {
        Ok(Vec::from(msg))
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxWalletResult<UnpackMessageOutput> {
        Ok(UnpackMessageOutput {
            message: format!("{:?}", msg),
            recipient_verkey: "".to_owned(),
            sender_verkey: None,
        })
    }
}
