use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
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
        Ok(utils::constants::VERKEY.to_string())
    }

    async fn replace_did_keys_start(&self, target_did: &str) -> VcxCoreResult<String> {
        Ok(utils::constants::VERKEY.to_string())
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

#[cfg(test)]
pub mod mocks {
    use aries_vcx_core::wallet::base_wallet::BaseWallet;
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub MockAllWallet {}

        impl BaseWallet for MockAllWallet {
            fn create_and_store_my_did<'life0,'life1,'life2,'async_trait>(&'life0 self,seed:Option< &'life1 str> ,method_name:Option< &'life2 str> ,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<(String,String)> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn key_for_local_did<'life0,'life1,'async_trait>(&'life0 self,did: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn replace_did_keys_start<'life0,'life1,'async_trait>(&'life0 self,target_did: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn replace_did_keys_apply<'life0,'life1,'async_trait>(&'life0 self,target_did: &'life1 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
            fn add_wallet_record<'life0,'life1,'life2,'life3,'life4,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str,value: &'life3 str,tags_json:Option< &'life4 str>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait;
            fn get_wallet_record<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str,options_json: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<String> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn delete_wallet_record<'life0,'life1,'life2,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn update_wallet_record_value<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str,value: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn add_wallet_record_tags<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str,tags_json: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn update_wallet_record_tags<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str,tags_json: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn delete_wallet_record_tags<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,xtype: &'life1 str,id: &'life2 str,tag_names: &'life3 str) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<()> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn iterate_wallet_records<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,xtype: &'life1 str,query: &'life2 str,options: &'life3 str,) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<Box<dyn aries_vcx_core::utils::async_fn_iterator::AsyncFnIterator<Item = aries_vcx_core::errors::error::VcxCoreResult<String> > > > > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn sign<'life0,'life1,'life2,'async_trait>(&'life0 self,my_vk: &'life1 str,msg: &'life2[u8]) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<Vec<u8> > > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,Self:'async_trait;
            fn verify<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,vk: &'life1 str,msg: &'life2[u8],signature: &'life3[u8]) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<bool> > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn pack_message<'life0,'life1,'life2,'life3,'async_trait>(&'life0 self,sender_vk:Option< &'life1 str> ,receiver_keys: &'life2 str,msg: &'life3[u8]) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<Vec<u8> > > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait;
            fn unpack_message<'life0,'life1,'async_trait>(&'life0 self,msg: &'life1[u8]) ->  core::pin::Pin<Box<dyn core::future::Future<Output = aries_vcx_core::errors::error::VcxCoreResult<Vec<u8> > > + core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait;
        }
    }
}