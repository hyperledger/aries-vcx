use std::{collections::HashMap, sync::Arc};

use futures::future::join;
use indy_api_types::errors::prelude::*;
use indy_utils::{
    crypto::{chacha20poly1305_ietf, hmacsha256},
    wql::Query,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{
    cache::wallet_cache::{WalletCache, WalletCacheHitMetrics},
    encryption::*,
    iterator::WalletIterator,
    query_encryption::encrypt_query,
    storage,
    storage::StorageRecord,
    RecordOptions, WalletRecord,
};

#[derive(Serialize, Deserialize)]
pub(super) struct Keys {
    pub type_key: chacha20poly1305_ietf::Key,
    pub name_key: chacha20poly1305_ietf::Key,
    pub value_key: chacha20poly1305_ietf::Key,
    pub item_hmac_key: hmacsha256::Key,
    pub tag_name_key: chacha20poly1305_ietf::Key,
    pub tag_value_key: chacha20poly1305_ietf::Key,
    pub tags_hmac_key: hmacsha256::Key,
}

impl Keys {
    pub fn new() -> Keys {
        Keys {
            type_key: chacha20poly1305_ietf::gen_key(),
            name_key: chacha20poly1305_ietf::gen_key(),
            value_key: chacha20poly1305_ietf::gen_key(),
            item_hmac_key: hmacsha256::gen_key(),
            tag_name_key: chacha20poly1305_ietf::gen_key(),
            tag_value_key: chacha20poly1305_ietf::gen_key(),
            tags_hmac_key: hmacsha256::gen_key(),
        }
    }

    pub fn serialize_encrypted(
        &self,
        master_key: &chacha20poly1305_ietf::Key,
    ) -> IndyResult<Vec<u8>> {
        let mut serialized = rmp_serde::to_vec(self)
            .to_indy(IndyErrorKind::InvalidState, "Unable to serialize keys")?;

        let encrypted = encrypt_as_not_searchable(&serialized, master_key);

        serialized.zeroize();
        Ok(encrypted)
    }

    pub fn deserialize_encrypted(
        bytes: &[u8],
        master_key: &chacha20poly1305_ietf::Key,
    ) -> IndyResult<Keys> {
        let mut decrypted = decrypt_merged(bytes, master_key)?;

        let keys: Keys = rmp_serde::from_slice(&decrypted)
            .to_indy(IndyErrorKind::InvalidState, "Invalid bytes for Key")?;

        decrypted.zeroize();
        Ok(keys)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedValue {
    pub data: Vec<u8>,
    pub key: Vec<u8>,
}

#[allow(dead_code)]
const ENCRYPTED_KEY_LEN: usize = chacha20poly1305_ietf::TAGBYTES
    + chacha20poly1305_ietf::NONCEBYTES
    + chacha20poly1305_ietf::KEYBYTES;

impl EncryptedValue {
    pub fn new(data: Vec<u8>, key: Vec<u8>) -> Self {
        Self { data, key }
    }

    pub fn encrypt(data: &str, key: &chacha20poly1305_ietf::Key) -> Self {
        let value_key = chacha20poly1305_ietf::gen_key();
        EncryptedValue::new(
            encrypt_as_not_searchable(data.as_bytes(), &value_key),
            encrypt_as_not_searchable(&value_key[..], key),
        )
    }

    pub fn decrypt(&self, key: &chacha20poly1305_ietf::Key) -> IndyResult<String> {
        let mut value_key_bytes = decrypt_merged(&self.key, key)?;

        let value_key = chacha20poly1305_ietf::Key::from_slice(&value_key_bytes)
            .map_err(|err| err.extend("Invalid value key"))?; // FIXME: review kind

        value_key_bytes.zeroize();

        let res = String::from_utf8(decrypt_merged(&self.data, &value_key)?).to_indy(
            IndyErrorKind::InvalidState,
            "Invalid UTF8 string inside of value",
        )?;

        Ok(res)
    }

    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.key.clone();
        result.extend_from_slice(self.data.as_slice());
        result
    }

    #[allow(dead_code)]
    pub fn from_bytes(joined_data: &[u8]) -> IndyResult<Self> {
        // value_key is stored as NONCE || CYPHERTEXT. Lenth of CYPHERTHEXT is length of DATA +
        // length of TAG.
        if joined_data.len() < ENCRYPTED_KEY_LEN {
            return Err(err_msg(
                IndyErrorKind::InvalidStructure,
                "Unable to split value_key from value: value too short",
            )); // FIXME: review kind
        }

        let value_key = joined_data[..ENCRYPTED_KEY_LEN].to_owned();
        let value = joined_data[ENCRYPTED_KEY_LEN..].to_owned();
        Ok(EncryptedValue {
            data: value,
            key: value_key,
        })
    }
}

pub(super) struct Wallet {
    id: String,
    storage: Box<dyn storage::WalletStorage>,
    keys: Arc<Keys>,
    cache: WalletCache,
}

impl Wallet {
    pub fn new(
        id: String,
        storage: Box<dyn storage::WalletStorage>,
        keys: Arc<Keys>,
        cache: WalletCache,
    ) -> Wallet {
        Wallet {
            id,
            storage,
            keys,
            cache,
        }
    }

    pub async fn add(
        &self,
        type_: &str,
        name: &str,
        value: &str,
        tags: &HashMap<String, String>,
        cache_record: bool,
    ) -> IndyResult<()> {
        let etype = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let ename = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        let evalue = EncryptedValue::encrypt(value, &self.keys.value_key);

        let etags = encrypt_tags(
            tags,
            &self.keys.tag_name_key,
            &self.keys.tag_value_key,
            &self.keys.tags_hmac_key,
        );

        self.storage.add(&etype, &ename, &evalue, &etags).await?;
        if cache_record {
            self.cache.add(type_, &etype, &ename, &evalue, &etags);
        }

        Ok(())
    }

    pub async fn add_tags(
        &self,
        type_: &str,
        name: &str,
        tags: &HashMap<String, String>,
    ) -> IndyResult<()> {
        let encrypted_type = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_name = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_tags = encrypt_tags(
            tags,
            &self.keys.tag_name_key,
            &self.keys.tag_value_key,
            &self.keys.tags_hmac_key,
        );

        self.storage
            .add_tags(&encrypted_type, &encrypted_name, &encrypted_tags)
            .await?;
        self.cache
            .add_tags(type_, &encrypted_type, &encrypted_name, &encrypted_tags)
            .await;

        Ok(())
    }

    pub async fn update_tags(
        &self,
        type_: &str,
        name: &str,
        tags: &HashMap<String, String>,
    ) -> IndyResult<()> {
        let encrypted_type = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_name = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_tags = encrypt_tags(
            tags,
            &self.keys.tag_name_key,
            &self.keys.tag_value_key,
            &self.keys.tags_hmac_key,
        );

        self.storage
            .update_tags(&encrypted_type, &encrypted_name, &encrypted_tags)
            .await?;
        self.cache
            .update_tags(type_, &encrypted_type, &encrypted_name, &encrypted_tags)
            .await;

        Ok(())
    }

    pub async fn delete_tags(&self, type_: &str, name: &str, tag_names: &[&str]) -> IndyResult<()> {
        let encrypted_type = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_name = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_tag_names =
            encrypt_tag_names(tag_names, &self.keys.tag_name_key, &self.keys.tags_hmac_key);

        self.storage
            .delete_tags(&encrypted_type, &encrypted_name, &encrypted_tag_names[..])
            .await?;
        self.cache
            .delete_tags(
                type_,
                &encrypted_type,
                &encrypted_name,
                &encrypted_tag_names[..],
            )
            .await;

        Ok(())
    }

    pub async fn update(&self, type_: &str, name: &str, new_value: &str) -> IndyResult<()> {
        let encrypted_type = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_name = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        let encrypted_value = EncryptedValue::encrypt(new_value, &self.keys.value_key);

        self.storage
            .update(&encrypted_type, &encrypted_name, &encrypted_value)
            .await?;
        self.cache
            .update(type_, &encrypted_type, &encrypted_name, &encrypted_value)
            .await;

        Ok(())
    }

    pub async fn get(
        &self,
        type_: &str,
        name: &str,
        options: &str,
        cache_hit_metrics: &WalletCacheHitMetrics,
    ) -> IndyResult<WalletRecord> {
        let etype = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let ename = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        let result = if self.cache.is_type_cacheable(type_) {
            let record_options: RecordOptions = serde_json::from_str(options).to_indy(
                IndyErrorKind::InvalidStructure,
                "RecordOptions is malformed json",
            )?;

            match self.cache.get(type_, &etype, &ename, &record_options).await {
                Some(result) => {
                    cache_hit_metrics.inc_cache_hit(type_).await;
                    result
                }
                None => {
                    // no item in cache, lets retrieve it and put it in cache.
                    let metrics_fut = cache_hit_metrics.inc_cache_miss(type_);
                    let full_options = RecordOptions {
                        retrieve_type: record_options.retrieve_type,
                        retrieve_value: true,
                        retrieve_tags: true,
                    };

                    let full_options = serde_json::to_string(&full_options).unwrap();

                    let storage_fut = self.storage.get(&etype, &ename, &full_options);
                    // run these two futures in parallel.
                    let full_result = join(storage_fut, metrics_fut).await.0?;

                    // save to cache only if valid data is returned (this should be always true).
                    if let (Some(evalue), Some(etags)) = (&full_result.value, &full_result.tags) {
                        self.cache.add(type_, &etype, &ename, evalue, etags);
                    }
                    StorageRecord {
                        id: full_result.id,
                        type_: if record_options.retrieve_type {
                            Some(etype)
                        } else {
                            None
                        },
                        value: if record_options.retrieve_value {
                            full_result.value
                        } else {
                            None
                        },
                        tags: if record_options.retrieve_tags {
                            full_result.tags
                        } else {
                            None
                        },
                    }
                }
            }
        } else {
            let metrics_fut = cache_hit_metrics.inc_not_cached(type_);
            let storage_fut = self.storage.get(&etype, &ename, options);
            // run these two futures in parallel.
            join(storage_fut, metrics_fut).await.0?
        };

        let value = match result.value {
            None => None,
            Some(encrypted_value) => Some(encrypted_value.decrypt(&self.keys.value_key)?),
        };

        let tags = decrypt_tags(
            &result.tags,
            &self.keys.tag_name_key,
            &self.keys.tag_value_key,
        )?;

        Ok(WalletRecord::new(
            String::from(name),
            result.type_.map(|_| type_.to_string()),
            value,
            tags,
        ))
    }

    pub async fn delete(&self, type_: &str, name: &str) -> IndyResult<()> {
        let etype = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let ename = encrypt_as_searchable(
            name.as_bytes(),
            &self.keys.name_key,
            &self.keys.item_hmac_key,
        );

        self.storage.delete(&etype, &ename).await?;
        self.cache.delete(type_, &etype, &ename).await;

        Ok(())
    }

    pub async fn search(
        &self,
        type_: &str,
        query: &str,
        options: Option<&str>,
    ) -> IndyResult<WalletIterator> {
        let parsed_query: Query = ::serde_json::from_str::<Query>(query)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::WalletQueryError, err))?
            .optimise()
            .unwrap_or_default();

        let encrypted_query = encrypt_query(parsed_query, &self.keys)?;

        let encrypted_type_ = encrypt_as_searchable(
            type_.as_bytes(),
            &self.keys.type_key,
            &self.keys.item_hmac_key,
        );

        let storage_iterator = self
            .storage
            .search(&encrypted_type_, &encrypted_query, options)
            .await?;

        let wallet_iterator = WalletIterator::new(storage_iterator, Arc::clone(&self.keys));

        Ok(wallet_iterator)
    }

    fn close(&mut self) -> IndyResult<()> {
        self.storage.close()
    }

    pub async fn get_all(&self) -> IndyResult<WalletIterator> {
        let all_items = self.storage.get_all().await?;
        Ok(WalletIterator::new(all_items, self.keys.clone()))
    }

    pub fn get_id<'a>(&'a self) -> &'a str {
        &self.id
    }
}

impl Drop for Wallet {
    fn drop(&mut self) {
        self.close().unwrap(); //FIXME pass the error to the API cb
    }
}
