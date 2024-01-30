use std::sync::Arc;

use agency_client::{
    errors::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult},
    wallet::base_agency_client_wallet::BaseAgencyClientWallet,
};
use async_trait::async_trait;
use public_key::{Key, KeyType};

use super::{
    base_wallet::{
        did_data::DidData,
        did_wallet::DidWallet,
        issuer_config::IssuerConfig,
        record::{AllRecords, Record},
        record_wallet::RecordWallet,
        search_filter::SearchFilter,
        BaseWallet,
    },
    structs_io::UnpackMessageOutput,
};
use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::entry_tag::EntryTags,
};

#[derive(Debug)]
pub struct AgencyClientWallet {
    inner: Arc<dyn BaseAgencyClientWallet>,
}

#[allow(unused_variables)]
#[async_trait]
impl BaseWallet for AgencyClientWallet {
    async fn export_wallet(&self, path: &str, backup_key: &str) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method("export_wallet"))
    }

    async fn close_wallet(&self) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method("close_wallet"))
    }

    async fn configure_issuer(&self, key_seed: &str) -> VcxCoreResult<IssuerConfig> {
        Err(unimplemented_agency_client_wallet_method(
            "configure_issuer",
        ))
    }

    // async fn create_wallet(wallet_config: WalletConfig) -> VcxCoreResult<Box<dyn BaseWallet>>
    // where
    //     Self: Sized,
    // {
    //     Err(unimplemented_agency_client_wallet_method("create_wallet"))
    // }

    // async fn open_wallet(wallet_config: &WalletConfig) -> VcxCoreResult<Box<dyn BaseWallet>>
    // where
    //     Self: Sized,
    // {
    //     Err(unimplemented_agency_client_wallet_method("open_wallet"))
    // }

    async fn all(&self) -> VcxCoreResult<Box<dyn AllRecords + Send>> {
        Err(unimplemented_agency_client_wallet_method("get_all"))
    }
}

#[allow(unused_variables)]
#[async_trait]
impl RecordWallet for AgencyClientWallet {
    async fn add_record(&self, record: Record) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method("add_record"))
    }

    async fn get_record(&self, category: &str, name: &str) -> VcxCoreResult<Record> {
        Err(unimplemented_agency_client_wallet_method("get_record"))
    }

    async fn update_record_tags(
        &self,
        category: &str,
        name: &str,
        new_tags: EntryTags,
    ) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method(
            "update_record_tags",
        ))
    }

    async fn update_record_value(
        &self,
        category: &str,
        name: &str,
        new_value: &str,
    ) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method(
            "update_record_value",
        ))
    }

    async fn delete_record(&self, category: &str, name: &str) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method("delete_record"))
    }

    async fn search_record(
        &self,
        category: &str,
        search_filter: Option<SearchFilter>,
    ) -> VcxCoreResult<Vec<Record>> {
        Err(unimplemented_agency_client_wallet_method("search_record"))
    }
}

#[async_trait]
#[allow(unused_variables)]
impl DidWallet for AgencyClientWallet {
    async fn create_and_store_my_did(
        &self,
        seed: Option<&str>,
        method_name: Option<&str>,
    ) -> VcxCoreResult<DidData> {
        Err(unimplemented_agency_client_wallet_method(
            "create_and_store_my_did",
        ))
    }

    async fn key_for_did(&self, name: &str) -> VcxCoreResult<Key> {
        Err(unimplemented_agency_client_wallet_method("key_for_did"))
    }

    async fn replace_did_key_start(&self, did: &str, seed: Option<&str>) -> VcxCoreResult<Key> {
        Err(unimplemented_agency_client_wallet_method(
            "replace_did_key_start",
        ))
    }

    async fn replace_did_key_apply(&self, did: &str) -> VcxCoreResult<()> {
        Err(unimplemented_agency_client_wallet_method(
            "replace_did_key_apply",
        ))
    }

    async fn sign(&self, key: &Key, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
        Err(unimplemented_agency_client_wallet_method("sign"))
    }

    async fn verify(&self, key: &Key, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
        Err(unimplemented_agency_client_wallet_method("verify"))
    }

    async fn pack_message(
        &self,
        sender_vk: Option<Key>,
        receiver_keys: Vec<Key>,
        msg: &[u8],
    ) -> VcxCoreResult<Vec<u8>> {
        let receiver_list: Vec<String> =
            receiver_keys.into_iter().map(|key| key.base58()).collect();

        let list_json = serde_json::to_string(&receiver_list)?;

        let res = self
            .inner
            .pack_message(
                sender_vk.map(|key| key.base58()).as_deref(),
                &list_json,
                msg,
            )
            .await?;

        Ok(res)
    }

    async fn unpack_message(&self, msg: &[u8]) -> VcxCoreResult<UnpackMessageOutput> {
        let unpack_json_bytes = self.inner.unpack_message(msg).await?;
        serde_json::from_slice(&unpack_json_bytes[..]).map_err(|err| {
            AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ParsingError, err.to_string())
        })
    }
}

pub trait ToBaseWallet {
    fn to_base_wallet(&self) -> AgencyClientWallet;
}

impl ToBaseWallet for Arc<dyn BaseAgencyClientWallet> {
    fn to_base_wallet(&self) -> AgencyClientWallet {
        AgencyClientWallet {
            inner: Arc::clone(self),
        }
    }
}

fn unimplemented_agency_client_wallet_method(method_name: &str) -> AriesVcxCoreError {
    // should not occur with proper internal usage - [AgencyClientWallet] is not public
    AriesVcxCoreError::from_msg(
        AriesVcxCoreErrorKind::UnimplementedFeature,
        format!("AgencyClientWallet::{method_name} is not intended to be used."),
    )
}

#[derive(Debug)]
pub(crate) struct BaseWalletAgencyClientWallet {
    inner: Arc<dyn BaseWallet>,
}

/// Implementation of [BaseAgencyClientWallet] which wraps over an [BaseWallet] implementation
/// to allow conversion
#[async_trait]
impl BaseAgencyClientWallet for BaseWalletAgencyClientWallet {
    async fn pack_message(
        &self,
        sender_vk: Option<&str>,
        receiver_keys: &str,
        msg: &[u8],
    ) -> AgencyClientResult<Vec<u8>> {
        let receiver_list = serde_json::from_str::<Vec<String>>(receiver_keys).map_err(|e| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::UnknownError,
                format!("A VCXError occured while calling pack_message: {e:?}"),
            )
        })?;

        let keys = receiver_list
            .into_iter()
            .map(|item| {
                Key::from_base58(&item, KeyType::Ed25519).map_err(|e| {
                    AgencyClientError::from_msg(
                        AgencyClientErrorKind::NotBase58,
                        format!("Invalid receiver key: {e:?}"),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sender_key = sender_vk
            .map(|item| {
                Key::from_base58(item, KeyType::Ed25519).map_err(|e| {
                    AgencyClientError::from_msg(
                        AgencyClientErrorKind::NotBase58,
                        format!("Invalid receiver key: {e:?}"),
                    )
                })
            })
            .transpose()?;

        self.inner
            .pack_message(sender_key, keys, msg)
            .await
            .map_err(|e| {
                AgencyClientError::from_msg(
                    AgencyClientErrorKind::UnknownError,
                    format!("A VCXError occured while calling pack_message: {e:?}"),
                )
            })
    }

    async fn unpack_message(&self, msg: &[u8]) -> AgencyClientResult<Vec<u8>> {
        let unpack = self.inner.unpack_message(msg).await.map_err(|e| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::UnknownError,
                format!("A VCXError occured while calling unpack_message: {e:?}"),
            )
        })?;
        serde_json::to_vec(&unpack).map_err(|err| {
            AgencyClientError::from_msg(
                AgencyClientErrorKind::UnknownError,
                format!("A VCXError occured while calling unpack_message: {err:?}"),
            )
        })
    }
}

pub trait ToBaseAgencyClientWallet {
    fn to_base_agency_client_wallet(&self) -> Arc<dyn BaseAgencyClientWallet>;
}

impl ToBaseAgencyClientWallet for Arc<dyn BaseWallet> {
    fn to_base_agency_client_wallet(&self) -> Arc<dyn BaseAgencyClientWallet> {
        let x = self.clone();
        Arc::new(BaseWalletAgencyClientWallet { inner: x })
    }
}
