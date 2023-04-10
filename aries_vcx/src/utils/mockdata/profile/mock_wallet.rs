use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
use aries_vcx_core::indy::utils::mocks::did_mocks::DidMocks;
use aries_vcx_core::utils::async_fn_iterator::AsyncFnIterator;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use async_trait::async_trait;

use crate::utils::{self};

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
    ) -> VcxCoreResult<(String, String)> {
        Ok((utils::constants::DID.to_string(), utils::constants::VERKEY.to_string()))
    }

    async fn key_for_local_did(&self, did: &str) -> VcxCoreResult<String> {
        get_next_mock_did_response_or_fail()
    }

    async fn replace_did_keys_start(&self, target_did: &str) -> VcxCoreResult<String> {
        get_next_mock_did_response_or_fail()
    }

    async fn replace_did_keys_apply(&self, target_did: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn add_wallet_record(
        &self,
        xtype: &str,
        id: &str,
        value: &str,
        tags_json: Option<&str>,
    ) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn get_wallet_record(&self, xtype: &str, id: &str, options_json: &str) -> VcxCoreResult<String> {
        Ok(r#"{"id":"123","type":"record type","value":"record value","tags":null}"#.to_string())
    }

    async fn delete_wallet_record(&self, xtype: &str, id: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn update_wallet_record_value(&self, xtype: &str, id: &str, value: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn add_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn update_wallet_record_tags(&self, xtype: &str, id: &str, tags_json: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn delete_wallet_record_tags(&self, xtype: &str, id: &str, tag_names: &str) -> VcxCoreResult<()> {
        Ok(())
    }

    async fn iterate_wallet_records(
        &self,
        xtype: &str,
        query: &str,
        options: &str,
    ) -> VcxCoreResult<Box<dyn AsyncFnIterator<Item = VcxCoreResult<String>>>> {
        // not needed yet
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "unimplemented mock method: iterate_wallet_records",
        ))
    }

    async fn sign(&self, my_vk: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Ok(Vec::from(msg))
    }

    async fn verify(&self, vk: &str, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        Ok(true)
    }

    async fn pack_message(&self, sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Ok(msg.to_vec())
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Ok(msg.to_vec())
    }
}

fn get_next_mock_did_response_or_fail() -> VcxCoreResult<String> {
    if DidMocks::has_did_mock_responses() {
        warn!("key_for_local_did >> retrieving did mock response");
        Ok(DidMocks::get_next_did_response())
    } else {
        Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnimplementedFeature,
            "DidMocks data for must be set",
        ))
    }
}
