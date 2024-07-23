// allow all clippy warnings, given this is legacy to be removed soon
#![allow(clippy::all)]
use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
    unimplemented,
};

use indy_api_types::{
    domain::wallet::{CacheConfig, Config, Credentials, ExportConfig, Tags},
    errors::prelude::*,
    WalletHandle,
};
use indy_utils::{
    crypto::chacha20poly1305_ietf::{self, Key as MasterKey},
    secret,
};
use iterator::WalletIterator;
use log::trace;
use serde::{Deserialize, Serialize};
use serde_json::Value as SValue;

pub use crate::encryption::KeyDerivationData;
use crate::{
    cache::wallet_cache::{WalletCache, WalletCacheHitData, WalletCacheHitMetrics},
    export_import::{export_continue, finish_import, preparse_file_to_import},
    storage::{
        default::SQLiteStorageType, mysql::MySqlStorageType, WalletStorage, WalletStorageType,
    },
    wallet::{Keys, Wallet},
};

mod encryption;
pub mod iterator;
mod query_encryption;
mod storage;

// TODO: Remove query language out of wallet module
pub mod language;

mod cache;
mod export_import;
mod wallet;

#[allow(clippy::type_complexity)]
pub struct WalletService {
    storage_types: Mutex<HashMap<String, Arc<dyn WalletStorageType>>>,
    wallets: Mutex<HashMap<WalletHandle, Arc<Wallet>>>,
    wallet_ids: Mutex<HashSet<String>>,
    pending_for_open: Mutex<
        HashMap<
            WalletHandle,
            (
                String, /* id */
                Box<dyn WalletStorage>,
                Metadata,
                Option<KeyDerivationData>,
            ),
        >,
    >,
    pending_for_import: Mutex<
        HashMap<
            WalletHandle,
            (
                BufReader<::std::fs::File>,
                chacha20poly1305_ietf::Nonce,
                usize,
                Vec<u8>,
                KeyDerivationData,
            ),
        >,
    >,
    cache_hit_metrics: WalletCacheHitMetrics,
}

#[allow(clippy::new_without_default)]
impl WalletService {
    pub fn new() -> WalletService {
        let storage_types = {
            let s1: Arc<dyn WalletStorageType> = Arc::new(SQLiteStorageType::new());
            let s2: Arc<dyn WalletStorageType> = Arc::new(MySqlStorageType::new());

            Mutex::new(HashMap::from([
                ("default".to_string(), s1),
                ("mysql".to_string(), s2),
            ]))
        };

        WalletService {
            storage_types,
            wallets: Mutex::new(HashMap::new()),
            wallet_ids: Mutex::new(HashSet::new()),
            pending_for_open: Mutex::new(HashMap::new()),
            pending_for_import: Mutex::new(HashMap::new()),
            cache_hit_metrics: WalletCacheHitMetrics::new(),
        }
    }

    pub async fn create_wallet(
        &self,
        config: &Config,
        credentials: &Credentials,
        key: (&KeyDerivationData, &MasterKey),
    ) -> IndyResult<()> {
        self._create_wallet(config, credentials, key).await?;
        Ok(())
    }

    async fn _create_wallet(
        &self,
        config: &Config,
        credentials: &Credentials,
        (key_data, master_key): (&KeyDerivationData, &MasterKey),
    ) -> IndyResult<Keys> {
        trace!(
            "create_wallet >>> config: {:?}, credentials: {:?}",
            config,
            secret!(credentials)
        );

        let keys = Keys::new();
        let metadata = self._prepare_metadata(master_key, key_data, &keys)?;

        let (storage_type, storage_config, storage_credentials) =
            self._get_config_and_cred_for_storage(config, credentials)?;

        storage_type
            .create_storage(
                &config.id,
                storage_config.as_deref(),
                storage_credentials.as_deref(),
                &metadata,
            )
            .await?;

        Ok(keys)
    }

    pub async fn delete_wallet_prepare(
        &self,
        config: &Config,
        credentials: &Credentials,
    ) -> IndyResult<(Metadata, KeyDerivationData)> {
        trace!(
            "delete_wallet >>> config: {:?}, credentials: {:?}",
            config,
            secret!(credentials)
        );

        if self
            .wallet_ids
            .lock()
            .unwrap()
            .contains(&WalletService::_get_wallet_id(config))
        {
            return Err(err_msg(
                IndyErrorKind::InvalidState,
                format!(
                    "Wallet has to be closed before deleting: {:?}",
                    WalletService::_get_wallet_id(config)
                ),
            ));
        }

        // check credentials and close connection before deleting wallet

        let (_, metadata, key_derivation_data) = self
            ._open_storage_and_fetch_metadata(config, credentials)
            .await?;

        Ok((metadata, key_derivation_data))
    }

    pub async fn delete_wallet_continue(
        &self,
        config: &Config,
        credentials: &Credentials,
        metadata: &Metadata,
        master_key: &MasterKey,
    ) -> IndyResult<()> {
        trace!(
            "delete_wallet >>> config: {:?}, credentials: {:?}",
            config,
            secret!(credentials)
        );

        {
            self._restore_keys(metadata, master_key)?;
        }

        let (storage_type, storage_config, storage_credentials) =
            self._get_config_and_cred_for_storage(config, credentials)?;

        storage_type
            .delete_storage(
                &config.id,
                storage_config.as_deref(),
                storage_credentials.as_deref(),
            )
            .await?;

        trace!("delete_wallet <<<");
        Ok(())
    }

    pub async fn open_wallet_prepare(
        &self,
        config: &Config,
        credentials: &Credentials,
    ) -> IndyResult<(WalletHandle, KeyDerivationData, Option<KeyDerivationData>)> {
        trace!(
            "open_wallet >>> config: {:?}, credentials: {:?}",
            config,
            secret!(&credentials)
        );

        self._is_id_from_config_not_used(config)?;

        let (storage, metadata, key_derivation_data) = self
            ._open_storage_and_fetch_metadata(config, credentials)
            .await?;

        let wallet_handle = indy_utils::next_wallet_handle();

        let rekey_data: Option<KeyDerivationData> = credentials.rekey.as_ref().map(|rekey| {
            KeyDerivationData::from_passphrase_with_new_salt(
                rekey,
                &credentials.rekey_derivation_method,
            )
        });

        self.pending_for_open.lock().unwrap().insert(
            wallet_handle,
            (
                WalletService::_get_wallet_id(config),
                storage,
                metadata,
                rekey_data.clone(),
            ),
        );

        Ok((wallet_handle, key_derivation_data, rekey_data))
    }

    pub async fn open_wallet_continue(
        &self,
        wallet_handle: WalletHandle,
        master_key: (&MasterKey, Option<&MasterKey>),
        cache_config: Option<CacheConfig>,
    ) -> IndyResult<WalletHandle> {
        let (id, storage, metadata, rekey_data) = self
            .pending_for_open
            .lock()
            .unwrap()
            .remove(&wallet_handle)
            .ok_or_else(|| err_msg(IndyErrorKind::InvalidState, "Open data not found"))?;

        let (master_key, rekey) = master_key;
        let keys = self._restore_keys(&metadata, master_key)?;

        // Rotate master key
        if let (Some(rekey), Some(rekey_data)) = (rekey, rekey_data) {
            let metadata = self._prepare_metadata(rekey, &rekey_data, &keys)?;
            storage.set_storage_metadata(&metadata).await?;
        }

        let wallet = Wallet::new(
            id.clone(),
            storage,
            Arc::new(keys),
            WalletCache::new(cache_config),
        );

        self.wallets
            .lock()
            .unwrap()
            .insert(wallet_handle, Arc::new(wallet));

        self.wallet_ids.lock().unwrap().insert(id.to_string());

        trace!("open_wallet <<< res: {:?}", wallet_handle);

        Ok(wallet_handle)
    }

    async fn _open_storage_and_fetch_metadata(
        &self,
        config: &Config,
        credentials: &Credentials,
    ) -> IndyResult<(Box<dyn WalletStorage>, Metadata, KeyDerivationData)> {
        let storage = self._open_storage(config, credentials).await?;

        let metadata: Metadata = {
            let metadata = storage.get_storage_metadata().await?;

            serde_json::from_slice(&metadata)
                .to_indy(IndyErrorKind::InvalidState, "Cannot deserialize metadata")?
        };

        let key_derivation_data = KeyDerivationData::from_passphrase_and_metadata(
            &credentials.key,
            &metadata,
            &credentials.key_derivation_method,
        )?;

        Ok((storage, metadata, key_derivation_data))
    }

    pub async fn close_wallet(&self, handle: WalletHandle) -> IndyResult<()> {
        trace!("close_wallet >>> handle: {:?}", handle);

        let wallet = self.wallets.lock().unwrap().remove(&handle);

        let wallet = if let Some(wallet) = wallet {
            wallet
        } else {
            return Err(err_msg(
                IndyErrorKind::InvalidWalletHandle,
                "Unknown wallet handle",
            ));
        };

        self.wallet_ids.lock().unwrap().remove(wallet.get_id());

        trace!("close_wallet <<<");

        Ok(())
    }

    fn _map_wallet_storage_error(err: IndyError, type_: &str, name: &str) -> IndyError {
        match err.kind() {
            IndyErrorKind::WalletItemAlreadyExists => err_msg(
                IndyErrorKind::WalletItemAlreadyExists,
                format!(
                    "Wallet item already exists with type: {}, id: {}",
                    type_, name
                ),
            ),
            IndyErrorKind::WalletItemNotFound => err_msg(
                IndyErrorKind::WalletItemNotFound,
                format!("Wallet item not found with type: {}, id: {}", type_, name),
            ),
            _ => err,
        }
    }

    pub async fn add_record(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
        value: &str,
        tags: &Tags,
    ) -> IndyResult<()> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .add(type_, name, value, tags, true)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn add_indy_record<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        value: &str,
        tags: &Tags,
    ) -> IndyResult<()>
    where
        T: Sized,
    {
        self.add_record(
            wallet_handle,
            &self.add_prefix(short_type_name::<T>()),
            name,
            value,
            tags,
        )
        .await?;

        Ok(())
    }

    pub async fn add_indy_object<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        object: &T,
        tags: &Tags,
    ) -> IndyResult<String>
    where
        T: ::serde::Serialize + Sized,
    {
        let object_json = serde_json::to_string(object).to_indy(
            IndyErrorKind::InvalidState,
            format!("Cannot serialize {:?}", short_type_name::<T>()),
        )?;

        self.add_indy_record::<T>(wallet_handle, name, &object_json, tags)
            .await?;

        Ok(object_json)
    }

    pub async fn update_record_value(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
        value: &str,
    ) -> IndyResult<()> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .update(type_, name, value)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn update_indy_object<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        object: &T,
    ) -> IndyResult<String>
    where
        T: ::serde::Serialize + Sized,
    {
        let type_ = short_type_name::<T>();

        let wallet = self.get_wallet(wallet_handle).await?;

        let object_json = serde_json::to_string(object).to_indy(
            IndyErrorKind::InvalidState,
            format!("Cannot serialize {:?}", type_),
        )?;

        wallet
            .update(&self.add_prefix(type_), name, &object_json)
            .await?;

        Ok(object_json)
    }

    pub async fn add_record_tags(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
        tags: &Tags,
    ) -> IndyResult<()> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .add_tags(type_, name, tags)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn update_record_tags(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
        tags: &Tags,
    ) -> IndyResult<()> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .update_tags(type_, name, tags)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn delete_record_tags(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
        tag_names: &[&str],
    ) -> IndyResult<()> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .delete_tags(type_, name, tag_names)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn delete_record(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
    ) -> IndyResult<()> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .delete(type_, name)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn delete_indy_record<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
    ) -> IndyResult<()>
    where
        T: Sized,
    {
        self.delete_record(
            wallet_handle,
            &self.add_prefix(short_type_name::<T>()),
            name,
        )
        .await?;

        Ok(())
    }

    pub async fn get_record(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        name: &str,
        options_json: &str,
    ) -> IndyResult<WalletRecord> {
        let wallet = self.get_wallet(wallet_handle).await?;
        wallet
            .get(type_, name, options_json, &self.cache_hit_metrics)
            .await
            .map_err(|err| WalletService::_map_wallet_storage_error(err, type_, name))
    }

    pub async fn get_indy_record<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        options_json: &str,
    ) -> IndyResult<WalletRecord>
    where
        T: Sized,
    {
        self.get_record(
            wallet_handle,
            &self.add_prefix(short_type_name::<T>()),
            name,
            options_json,
        )
        .await
    }

    pub async fn get_indy_record_value<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        options_json: &str,
    ) -> IndyResult<String>
    where
        T: Sized,
    {
        let type_ = short_type_name::<T>();

        let record = self
            .get_record(wallet_handle, &self.add_prefix(type_), name, options_json)
            .await?;

        let record_value = record
            .get_value()
            .ok_or_else(|| {
                err_msg(
                    IndyErrorKind::InvalidState,
                    format!("{} not found for id: {:?}", type_, name),
                )
            })?
            .to_string();

        Ok(record_value)
    }

    // Dirty hack. json must live longer then result T
    pub async fn get_indy_object<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        options_json: &str,
    ) -> IndyResult<T>
    where
        T: ::serde::de::DeserializeOwned + Sized,
    {
        let record_value = self
            .get_indy_record_value::<T>(wallet_handle, name, options_json)
            .await?;

        serde_json::from_str(&record_value).to_indy(
            IndyErrorKind::InvalidState,
            format!("Cannot deserialize {:?}", short_type_name::<T>()),
        )
    }

    // Dirty hack. json must live longer then result T
    pub async fn get_indy_opt_object<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        options_json: &str,
    ) -> IndyResult<Option<T>>
    where
        T: ::serde::de::DeserializeOwned + Sized,
    {
        match self
            .get_indy_object::<T>(wallet_handle, name, options_json)
            .await
        {
            Ok(res) => Ok(Some(res)),
            Err(ref err) if err.kind() == IndyErrorKind::WalletItemNotFound => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn search_records(
        &self,
        wallet_handle: WalletHandle,
        type_: &str,
        query_json: &str,
        options_json: &str,
    ) -> IndyResult<WalletSearch> {
        let wallet = self.get_wallet(wallet_handle).await?;

        Ok(WalletSearch {
            iter: wallet.search(type_, query_json, Some(options_json)).await?,
        })
    }

    pub async fn search_indy_records<T>(
        &self,
        wallet_handle: WalletHandle,
        query_json: &str,
        options_json: &str,
    ) -> IndyResult<WalletSearch>
    where
        T: Sized,
    {
        self.search_records(
            wallet_handle,
            &self.add_prefix(short_type_name::<T>()),
            query_json,
            options_json,
        )
        .await
    }

    #[allow(dead_code)] // TODO: Should we implement getting all records or delete everywhere?
    pub fn search_all_records(&self, _wallet_handle: WalletHandle) -> IndyResult<WalletSearch> {
        //        match self.wallets.lock().await.get(&wallet_handle) {
        //            Some(wallet) => wallet.search_all_records(),
        //            None => Err(IndyError::InvalidHandle(wallet_handle.to_string()))
        //        }
        unimplemented!()
    }

    pub async fn upsert_indy_object<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
        object: &T,
    ) -> IndyResult<String>
    where
        T: ::serde::Serialize + Sized,
    {
        if self.record_exists::<T>(wallet_handle, name).await? {
            self.update_indy_object::<T>(wallet_handle, name, object)
                .await
        } else {
            self.add_indy_object::<T>(wallet_handle, name, object, &HashMap::new())
                .await
        }
    }

    pub async fn record_exists<T>(
        &self,
        wallet_handle: WalletHandle,
        name: &str,
    ) -> IndyResult<bool>
    where
        T: Sized,
    {
        match self
            .get_record(
                wallet_handle,
                &self.add_prefix(short_type_name::<T>()),
                name,
                &RecordOptions::id(),
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(ref err) if err.kind() == IndyErrorKind::WalletItemNotFound => Ok(false),
            Err(err) => Err(err),
        }
    }

    pub async fn check(&self, handle: WalletHandle) -> IndyResult<()> {
        self.get_wallet(handle).await?;
        Ok(())
    }

    pub async fn get_all(&self, handle: WalletHandle) -> IndyResult<WalletIterator> {
        let wallet = self.get_wallet(handle).await?;
        wallet.get_all().await
    }

    pub async fn export_wallet(
        &self,
        wallet_handle: WalletHandle,
        export_config: &ExportConfig,
        version: u32,
        key: (&KeyDerivationData, &MasterKey),
    ) -> IndyResult<()> {
        trace!(
            "export_wallet >>> wallet_handle: {:?}, export_config: {:?}, version: {:?}",
            wallet_handle,
            secret!(export_config),
            version
        );

        if version != 0 {
            return Err(err_msg(IndyErrorKind::InvalidState, "Unsupported version"));
        }

        let (key_data, key) = key;

        let wallet = self.get_wallet(wallet_handle).await?;

        let path = PathBuf::from(&export_config.path);

        if let Some(parent_path) = path.parent() {
            fs::DirBuilder::new().recursive(true).create(parent_path)?;
        }

        let mut export_file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(export_config.path.clone())?;

        let res = export_continue(wallet, &mut export_file, version, key.clone(), key_data).await;

        trace!("export_wallet <<<");
        res
    }

    pub async fn import_wallet_prepare(
        &self,
        config: &Config,
        credentials: &Credentials,
        export_config: &ExportConfig,
    ) -> IndyResult<(WalletHandle, KeyDerivationData, KeyDerivationData)> {
        trace!(
            "import_wallet_prepare >>> config: {:?}, credentials: {:?}, export_config: {:?}",
            config,
            secret!(export_config),
            secret!(export_config)
        );

        let exported_file_to_import = fs::OpenOptions::new()
            .read(true)
            .open(&export_config.path)?;

        let (reader, import_key_derivation_data, nonce, chunk_size, header_bytes) =
            preparse_file_to_import(exported_file_to_import, &export_config.key)?;
        let key_data = KeyDerivationData::from_passphrase_with_new_salt(
            &credentials.key,
            &credentials.key_derivation_method,
        );

        let wallet_handle = indy_utils::next_wallet_handle();

        let stashed_key_data = key_data.clone();

        self.pending_for_import.lock().unwrap().insert(
            wallet_handle,
            (reader, nonce, chunk_size, header_bytes, stashed_key_data),
        );

        Ok((wallet_handle, key_data, import_key_derivation_data))
    }

    pub async fn import_wallet_continue(
        &self,
        wallet_handle: WalletHandle,
        config: &Config,
        credentials: &Credentials,
        key: (MasterKey, MasterKey),
    ) -> IndyResult<()> {
        let (reader, nonce, chunk_size, header_bytes, key_data) = self
            .pending_for_import
            .lock()
            .unwrap()
            .remove(&wallet_handle)
            .unwrap();

        let (import_key, master_key) = key;

        let keys = self
            ._create_wallet(config, credentials, (&key_data, &master_key))
            .await?;

        self._is_id_from_config_not_used(config)?;

        let storage = self._open_storage(config, credentials).await?;
        let metadata = storage.get_storage_metadata().await?;

        let res = {
            let wallet = Wallet::new(
                WalletService::_get_wallet_id(config),
                storage,
                Arc::new(keys),
                WalletCache::new(None),
            );

            finish_import(&wallet, reader, import_key, nonce, chunk_size, header_bytes).await
        };

        if res.is_err() {
            let metadata: Metadata = serde_json::from_slice(&metadata)
                .to_indy(IndyErrorKind::InvalidState, "Cannot deserialize metadata")?;

            self.delete_wallet_continue(config, credentials, &metadata, &master_key)
                .await?;
        }

        //        self.close_wallet(wallet_handle)?;

        trace!("import_wallet <<<");
        res
    }

    pub fn get_wallets_count(&self) -> usize {
        self.wallets.lock().unwrap().len()
    }

    pub fn get_wallet_ids_count(&self) -> usize {
        self.wallet_ids.lock().unwrap().len()
    }

    pub fn get_pending_for_import_count(&self) -> usize {
        self.pending_for_import.lock().unwrap().len()
    }

    pub fn get_pending_for_open_count(&self) -> usize {
        self.pending_for_open.lock().unwrap().len()
    }

    pub async fn get_wallet_cache_hit_metrics_data(&self) -> HashMap<String, WalletCacheHitData> {
        self.cache_hit_metrics.get_data()
    }

    #[allow(clippy::type_complexity)]
    fn _get_config_and_cred_for_storage(
        &self,
        config: &Config,
        credentials: &Credentials,
    ) -> IndyResult<(Arc<dyn WalletStorageType>, Option<String>, Option<String>)> {
        let storage_type = {
            let storage_type = config.storage_type.as_deref().unwrap_or("default");

            self.storage_types
                .lock()
                .unwrap()
                .get(storage_type)
                .ok_or_else(|| {
                    err_msg(
                        IndyErrorKind::UnknownWalletStorageType,
                        "Unknown wallet storage type",
                    )
                })?
                .clone()
        };

        let storage_config = config.storage_config.as_ref().map(SValue::to_string);

        let storage_credentials = credentials
            .storage_credentials
            .as_ref()
            .map(SValue::to_string);

        Ok((storage_type, storage_config, storage_credentials))
    }

    fn _is_id_from_config_not_used(&self, config: &Config) -> IndyResult<()> {
        let id = WalletService::_get_wallet_id(config);
        if self.wallet_ids.lock().unwrap().contains(&id) {
            return Err(err_msg(
                IndyErrorKind::WalletAlreadyOpened,
                format!(
                    "Wallet {} already opened",
                    WalletService::_get_wallet_id(config)
                ),
            ));
        }

        Ok(())
    }

    fn _get_wallet_id(config: &Config) -> String {
        let wallet_path = config
            .storage_config
            .as_ref()
            .and_then(|storage_config| storage_config["path"].as_str())
            .unwrap_or("");

        format!("{}{}", config.id, wallet_path)
    }

    async fn _open_storage(
        &self,
        config: &Config,
        credentials: &Credentials,
    ) -> IndyResult<Box<dyn WalletStorage>> {
        let (storage_type, storage_config, storage_credentials) =
            self._get_config_and_cred_for_storage(config, credentials)?;

        let storage = storage_type
            .open_storage(
                &config.id,
                storage_config.as_deref(),
                storage_credentials.as_deref(),
            )
            .await?;

        Ok(storage)
    }

    fn _prepare_metadata(
        &self,
        master_key: &chacha20poly1305_ietf::Key,
        key_data: &KeyDerivationData,
        keys: &Keys,
    ) -> IndyResult<Vec<u8>> {
        let encrypted_keys = keys.serialize_encrypted(master_key)?;

        let metadata = match key_data {
            KeyDerivationData::Raw(_) => Metadata::MetadataRaw(MetadataRaw {
                keys: encrypted_keys,
            }),
            KeyDerivationData::Argon2iInt(_, salt) | KeyDerivationData::Argon2iMod(_, salt) => {
                Metadata::MetadataArgon(MetadataArgon {
                    keys: encrypted_keys,
                    master_key_salt: salt[..].to_vec(),
                })
            }
        };

        let res = serde_json::to_vec(&metadata).to_indy(
            IndyErrorKind::InvalidState,
            "Cannot serialize wallet metadata",
        )?;

        Ok(res)
    }

    fn _restore_keys(&self, metadata: &Metadata, master_key: &MasterKey) -> IndyResult<Keys> {
        let metadata_keys = metadata.get_keys();

        let res = Keys::deserialize_encrypted(metadata_keys, master_key).map_err(|err| {
            err.map(
                IndyErrorKind::WalletAccessFailed,
                "Invalid master key provided",
            )
        })?;

        Ok(res)
    }

    pub const PREFIX: &'static str = "Indy";

    pub fn add_prefix(&self, type_: &str) -> String {
        format!("{}::{}", WalletService::PREFIX, type_)
    }

    async fn get_wallet(&self, wallet_handle: WalletHandle) -> IndyResult<Arc<Wallet>> {
        let wallets = self.wallets.lock().unwrap(); //await;
        let w = wallets.get(&wallet_handle);
        if let Some(w) = w {
            Ok(w.clone())
        } else {
            Err(err_msg(
                IndyErrorKind::InvalidWalletHandle,
                "Unknown wallet handle",
            ))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Metadata {
    MetadataArgon(MetadataArgon),
    MetadataRaw(MetadataRaw),
}

impl Metadata {
    pub fn get_keys(&self) -> &Vec<u8> {
        match *self {
            Metadata::MetadataArgon(ref metadata) => &metadata.keys,
            Metadata::MetadataRaw(ref metadata) => &metadata.keys,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataArgon {
    pub keys: Vec<u8>,
    pub master_key_salt: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataRaw {
    pub keys: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletRecord {
    #[serde(rename = "type")]
    type_: Option<String>,
    id: String,
    value: Option<String>,
    tags: Option<Tags>,
}

impl fmt::Debug for WalletRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalletRecord")
            .field("type_", &self.type_)
            .field("id", &self.id)
            .field("value", &self.value.as_ref().map(|_| "******"))
            .field("tags", &self.tags)
            .finish()
    }
}

impl Ord for WalletRecord {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        (&self.type_, &self.id).cmp(&(&other.type_, &other.id))
    }
}

impl PartialOrd for WalletRecord {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl WalletRecord {
    pub fn new(
        name: String,
        type_: Option<String>,
        value: Option<String>,
        tags: Option<Tags>,
    ) -> WalletRecord {
        WalletRecord {
            id: name,
            type_,
            value,
            tags,
        }
    }

    pub fn get_id(&self) -> &str {
        self.id.as_str()
    }

    #[allow(dead_code)]
    pub fn get_type(&self) -> Option<&str> {
        self.type_.as_deref()
    }

    pub fn get_value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    #[allow(dead_code)]
    pub fn get_tags(&self) -> Option<&Tags> {
        self.tags.as_ref()
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordOptions {
    #[serde(default = "default_false")]
    retrieve_type: bool,
    #[serde(default = "default_true")]
    retrieve_value: bool,
    #[serde(default = "default_false")]
    retrieve_tags: bool,
}

impl RecordOptions {
    pub fn id() -> String {
        let options = RecordOptions {
            retrieve_type: false,
            retrieve_value: false,
            retrieve_tags: false,
        };

        serde_json::to_string(&options).unwrap()
    }

    pub fn id_value() -> String {
        let options = RecordOptions {
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: false,
        };

        serde_json::to_string(&options).unwrap()
    }

    pub fn id_value_tags() -> String {
        let options = RecordOptions {
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: true,
        };

        serde_json::to_string(&options).unwrap()
    }
}

impl Default for RecordOptions {
    fn default() -> RecordOptions {
        RecordOptions {
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: false,
        }
    }
}

pub struct WalletSearch {
    iter: iterator::WalletIterator,
}

impl WalletSearch {
    pub fn get_total_count(&self) -> IndyResult<Option<usize>> {
        self.iter.get_total_count()
    }

    pub async fn fetch_next_record(&mut self) -> IndyResult<Option<WalletRecord>> {
        self.iter.next().await
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    #[serde(default = "default_true")]
    retrieve_records: bool,
    #[serde(default = "default_false")]
    retrieve_total_count: bool,
    #[serde(default = "default_false")]
    retrieve_type: bool,
    #[serde(default = "default_true")]
    retrieve_value: bool,
    #[serde(default = "default_false")]
    retrieve_tags: bool,
}

impl SearchOptions {
    pub fn id_value() -> String {
        let options = SearchOptions {
            retrieve_records: true,
            retrieve_total_count: true,
            retrieve_type: true,
            retrieve_value: true,
            retrieve_tags: false,
        };

        serde_json::to_string(&options).unwrap()
    }
}

impl Default for SearchOptions {
    fn default() -> SearchOptions {
        SearchOptions {
            retrieve_records: true,
            retrieve_total_count: false,
            retrieve_type: false,
            retrieve_value: true,
            retrieve_tags: false,
        }
    }
}

fn short_type_name<T>() -> &'static str {
    let type_name = std::any::type_name::<T>();
    type_name.rsplit("::").next().unwrap_or(type_name)
}
