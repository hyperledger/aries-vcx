use async_trait::async_trait;

use crate::{
    indy::utils::mocks::did_mocks::DidMocks,
    plugins::wallet::base_wallet::BaseWallet,
    utils::{self, async_fn_iterator::AsyncFnIterator},
};
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[derive(Debug)]
pub(crate) struct MockWallet;

// NOTE : currently matches the expected results if did_mocks and indy_mocks are enabled
/// Implementation of [BaseAnoncreds] which responds with mock data
#[allow(unused)]
#[async_trait]
impl BaseWallet for MockWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxResult<(String, String)> {
        Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()))
    }

    async fn key_for_local_did(&self, did: &str) -> VcxResult<String> {
        get_next_mock_did_response_or_fail()
    }

    async fn replace_did_keys_start(&self, target_did: &str) -> VcxResult<String> {
        get_next_mock_did_response_or_fail()
    }

    async fn replace_did_keys_apply(&self, target_did: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn add_wallet_record(&self, xtype: &str, id: &str, value: &str, tags_json: Option<&str>) -> VcxResult<()> {
        Ok(())
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options_json: &str) -> VcxResult<String> {
        Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string())
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn add_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn update_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn delete_wallet_record_tags(&self, xtype: &str, id: &str, tag_names: &str) -> VcxResult<()> {
        Ok(())
    }

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxResult<Box<dyn AsyncFnIterator<Item = VcxResult<String>>>> {
        // not needed yet
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::UnimplementedFeature,
            "unimplemented mock method",
        ))
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        Ok(Vec::from(msg))
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
        Ok(true)
    }

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
        Ok(msg.to_vec())
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxResult<Vec<u8>> {
        Ok(msg.to_vec())
    }
}

fn get_next_mock_did_response_or_fail() -> VcxResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("key_for_local_did >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::UnimplementedFeature,
            "DidMocks data for must be set",
        ))
    }
}
