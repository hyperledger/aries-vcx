use std::sync::Arc;

// use async_std::task::spawn_blocking;
use indy_api_types::{
    domain::wallet::{Config, Credentials, ExportConfig, KeyConfig},
    errors::prelude::*,
    wallet::*,
    WalletHandle,
};
use indy_utils::crypto::{chacha20poly1305_ietf, chacha20poly1305_ietf::Key as MasterKey, randombytes};
use indy_wallet::{KeyDerivationData, WalletService};

use crate::{services::CryptoService, utils::crypto::base58::ToBase58};

pub struct WalletController {
    wallet_service: Arc<WalletService>,
    crypto_service: Arc<CryptoService>,
}

impl WalletController {
    pub(crate) fn new(wallet_service: Arc<WalletService>, crypto_service: Arc<CryptoService>) -> WalletController {
        WalletController {
            wallet_service,
            crypto_service,
        }
    }

    /// Register custom wallet storage implementation.
    ///
    /// #Params

    /// type_: Storage type name.
    /// create: WalletType create operation handler
    /// open: WalletType open operation handler
    /// close: Wallet close operation handler
    /// delete: WalletType delete operation handler
    /// add_record: WalletType add record operation handler
    /// update_record_value: WalletType update record value operation handler
    /// update_record_tags: WalletType update record tags operation handler
    /// add_record_tags: WalletType add record tags operation handler
    /// delete_record_tags: WalletType delete record tags operation handler
    /// delete_record: WalletType delete record operation handler
    /// get_record: WalletType get record operation handler
    /// get_record_id: WalletType get record id operation handler
    /// get_record_type: WalletType get record type operation handler
    /// get_record_value: WalletType get record value operation handler
    /// get_record_tags: WalletType get record tags operation handler
    /// free_record: WalletType free record operation handler
    /// search_records: WalletType search records operation handler
    /// search_all_records: WalletType search all records operation handler
    /// get_search_total_count: WalletType get search total count operation handler
    /// fetch_search_next_record: WalletType fetch search next record operation handler
    /// free_search: WalletType free search operation handler
    /// free: Handler that allows to de-allocate strings allocated in caller code
    ///
    /// #Returns
    /// Error code
    pub fn register_type(
        &self,
        type_: String,
        create: WalletCreate,
        open: WalletOpen,
        close: WalletClose,
        delete: WalletDelete,
        add_record: WalletAddRecord,
        update_record_value: WalletUpdateRecordValue,
        update_record_tags: WalletUpdateRecordTags,
        add_record_tags: WalletAddRecordTags,
        delete_record_tags: WalletDeleteRecordTags,
        delete_record: WalletDeleteRecord,
        get_record: WalletGetRecord,
        get_record_id: WalletGetRecordId,
        get_record_type: WalletGetRecordType,
        get_record_value: WalletGetRecordValue,
        get_record_tags: WalletGetRecordTags,
        free_record: WalletFreeRecord,
        get_storage_metadata: WalletGetStorageMetadata,
        set_storage_metadata: WalletSetStorageMetadata,
        free_storage_metadata: WalletFreeStorageMetadata,
        search_records: WalletSearchRecords,
        search_all_records: WalletSearchAllRecords,
        get_search_total_count: WalletGetSearchTotalCount,
        fetch_search_next_record: WalletFetchSearchNextRecord,
        free_search: WalletFreeSearch,
    ) -> IndyResult<()> {
        trace!("register_type > type_: {:?}", type_);

        self.wallet_service.register_wallet_storage(
            &type_,
            create,
            open,
            close,
            delete,
            add_record,
            update_record_value,
            update_record_tags,
            add_record_tags,
            delete_record_tags,
            delete_record,
            get_record,
            get_record_id,
            get_record_type,
            get_record_value,
            get_record_tags,
            free_record,
            get_storage_metadata,
            set_storage_metadata,
            free_storage_metadata,
            search_records,
            search_all_records,
            get_search_total_count,
            fetch_search_next_record,
            free_search,
        )?;

        trace!("register_type < res: ()");
        Ok(())
    }

    /// Create a new secure wallet.
    ///
    /// #Params
    /// config: Wallet configuration json.
    /// {
    ///   "id": string, Identifier of the wallet.
    ///         Configured storage uses this identifier to lookup exact wallet data placement.
    ///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
    ///                  'Default' storage type allows to store wallet data in the local file.
    ///                  Custom storage types can be registered with indy_register_wallet_storage
    /// call.   "storage_config": optional<object>, Storage configuration json. Storage type
    /// defines set of supported keys.                     Can be optional if storage supports
    /// default configuration.                     For 'default' storage type configuration is:
    ///   {
    ///     "path": optional<string>, Path to the directory with wallet files.
    ///             Defaults to $HOME/.indy_client/wallet.
    ///             Wallet will be stored in the file {path}/{id}/sqlite.db
    ///   }
    /// }
    /// credentials: Wallet credentials json
    /// {
    ///   "key": string, Key or passphrase used for wallet key derivation.
    ///                  Look to key_derivation_method param for information about supported key
    /// derivation methods.   "storage_credentials": optional<object> Credentials for wallet
    /// storage. Storage type defines set of supported keys.                          Can be
    /// optional if storage supports default configuration.                          For
    /// 'default' storage type should be empty.   "key_derivation_method": optional<string>
    /// Algorithm to use for wallet key derivation:                          ARGON2I_MOD -
    /// derive secured wallet master key (used by default)                          ARGON2I_INT
    /// - derive secured wallet master key (less secured but faster)                          
    /// RAW - raw wallet key master provided (skip derivation).                                
    /// RAW keys can be generated with indy_generate_wallet_key call }
    ///
    /// #Returns
    /// err: Error code
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub async fn create(&self, config: Config, credentials: Credentials) -> IndyResult<()> {
        trace!(
            "_create > config: {:?} credentials: {:?}",
            &config,
            secret!(&credentials)
        );

        let key_data =
            KeyDerivationData::from_passphrase_with_new_salt(&credentials.key, &credentials.key_derivation_method);

        let key = Self::_derive_key(&key_data).await?;

        let res = self
            .wallet_service
            .create_wallet(&config, &credentials, (&key_data, &key))
            .await;

        trace!("create < {:?}", res);
        res
    }

    /// Open the wallet.
    ///
    /// Wallet must be previously created with indy_create_wallet method.
    ///
    /// #Params
    /// config: Wallet configuration json.
    ///   {
    ///       "id": string, Identifier of the wallet.
    ///             Configured storage uses this identifier to lookup exact wallet data placement.
    ///       "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
    ///                       'Default' storage type allows to store wallet data in the local file.
    ///                       Custom storage types can be registered with
    /// indy_register_wallet_storage call.       "storage_config": optional<object>, Storage
    /// configuration json. Storage type defines set of supported keys.                         
    /// Can be optional if storage supports default configuration.                         For
    /// 'default' storage type configuration is:           {
    ///              "path": optional<string>, Path to the directory with wallet files.
    ///                      Defaults to $HOME/.indy_client/wallet.
    ///                      Wallet will be stored in the file {path}/{id}/sqlite.db
    ///           }
    ///       "cache": optional<object>, Cache configuration json. If omitted the cache is disabled
    /// (default).       {
    ///           "size": optional<int>, Number of items in cache,
    ///           "entities": List<string>, Types of items being cached. eg. ["vdrtools::Did",
    /// "vdrtools::Key"]           "algorithm" optional<string>, cache algorithm, defaults to
    /// lru, which is the only one supported for now.       }
    ///   }
    /// credentials: Wallet credentials json
    ///   {
    ///       "key": string, Key or passphrase used for wallet key derivation.
    ///                      Look to key_derivation_method param for information about supported key
    /// derivation methods.       "rekey": optional<string>, If present than wallet master key
    /// will be rotated to a new one.       "storage_credentials": optional<object> Credentials
    /// for wallet storage. Storage type defines set of supported keys.                         
    /// Can be optional if storage supports default configuration.                              
    /// For 'default' storage type should be empty.       "key_derivation_method":
    /// optional<string> Algorithm to use for wallet key derivation:                          
    /// ARGON2I_MOD - derive secured wallet master key (used by default)                        
    /// ARGON2I_INT - derive secured wallet master key (less secured but faster)                
    /// RAW - raw wallet key master provided (skip derivation).                                
    /// RAW keys can be generated with indy_generate_wallet_key call
    ///       "rekey_derivation_method": optional<string> Algorithm to use for wallet rekey
    /// derivation:                          ARGON2I_MOD - derive secured wallet master rekey
    /// (used by default)                          ARGON2I_INT - derive secured wallet master
    /// rekey (less secured but faster)                          RAW - raw wallet rekey master
    /// provided (skip derivation).                                RAW keys can be generated
    /// with indy_generate_wallet_key call   }
    ///
    /// #Returns
    /// err: Error code
    /// handle: Handle to opened wallet to use in methods that require wallet access.
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub async fn open(&self, config: Config, credentials: Credentials) -> IndyResult<WalletHandle> {
        trace!("open > config: {:?} credentials: {:?}", &config, secret!(&credentials));
        // TODO: try to refactor to avoid usage of continue methods

        let (wallet_handle, key_derivation_data, rekey_data) =
            self.wallet_service.open_wallet_prepare(&config, &credentials).await?;

        let key = Self::_derive_key(&key_derivation_data).await?;

        let rekey = if let Some(rekey_data) = rekey_data {
            Some(Self::_derive_key(&rekey_data).await?)
        } else {
            None
        };

        let res = self
            .wallet_service
            .open_wallet_continue(wallet_handle, (&key, rekey.as_ref()), config.cache)
            .await;

        trace!("open < res: {:?}", res);

        res
    }

    /// Closes opened wallet and frees allocated resources.
    ///
    /// #Params
    /// wallet_handle: wallet handle returned by indy_open_wallet.
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub async fn close(&self, wallet_handle: WalletHandle) -> IndyResult<()> {
        trace!("close > handle: {:?}", wallet_handle);

        self.wallet_service.close_wallet(wallet_handle).await?;

        trace!("close < res: ()");
        Ok(())
    }

    /// Deletes created wallet.
    ///
    /// #Params
    /// config: Wallet configuration json.
    /// {
    ///   "id": string, Identifier of the wallet.
    ///         Configured storage uses this identifier to lookup exact wallet data placement.
    ///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
    ///                  'Default' storage type allows to store wallet data in the local file.
    ///                  Custom storage types can be registered with indy_register_wallet_storage
    /// call.   "storage_config": optional<object>, Storage configuration json. Storage type
    /// defines set of supported keys.                     Can be optional if storage supports
    /// default configuration.                     For 'default' storage type configuration is:
    ///   {
    ///     "path": optional<string>, Path to the directory with wallet files.
    ///             Defaults to $HOME/.indy_client/wallet.
    ///             Wallet will be stored in the file {path}/{id}/sqlite.db
    ///   }
    /// }
    /// credentials: Wallet credentials json
    /// {
    ///   "key": string, Key or passphrase used for wallet key derivation.
    ///                  Look to key_derivation_method param for information about supported key
    /// derivation methods.   "storage_credentials": optional<object> Credentials for wallet
    /// storage. Storage type defines set of supported keys.                          Can be
    /// optional if storage supports default configuration.                          For
    /// 'default' storage type should be empty.   "key_derivation_method": optional<string>
    /// Algorithm to use for wallet key derivation:                             ARGON2I_MOD -
    /// derive secured wallet master key (used by default)                             
    /// ARGON2I_INT - derive secured wallet master key (less secured but faster)                
    /// RAW - raw wallet key master provided (skip derivation).                                
    /// RAW keys can be generated with indy_generate_wallet_key call }
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub async fn delete(&self, config: Config, credentials: Credentials) -> IndyResult<()> {
        trace!(
            "delete > config: {:?} credentials: {:?}",
            &config,
            secret!(&credentials)
        );
        // TODO: try to refactor to avoid usage of continue methods

        let (metadata, key_derivation_data) = self.wallet_service.delete_wallet_prepare(&config, &credentials).await?;

        let key = Self::_derive_key(&key_derivation_data).await?;

        let res = self
            .wallet_service
            .delete_wallet_continue(&config, &credentials, &metadata, &key)
            .await;

        trace!("delete < {:?}", res);
        res
    }

    /// Exports opened wallet
    ///
    /// #Params:
    /// wallet_handle: wallet handle returned by indy_open_wallet
    /// export_config: JSON containing settings for input operation.
    ///   {
    ///     "path": <string>, Path of the file that contains exported wallet content
    ///     "key": <string>, Key or passphrase used for wallet export key derivation.
    ///                     Look to key_derivation_method param for information about supported key
    /// derivation methods.     "key_derivation_method": optional<string> Algorithm to use for
    /// wallet export key derivation:                              ARGON2I_MOD - derive secured
    /// export key (used by default)                              ARGON2I_INT - derive secured
    /// export key (less secured but faster)                              RAW - raw export key
    /// provided (skip derivation).                                RAW keys can be generated
    /// with indy_generate_wallet_key call   }
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub async fn export(&self, wallet_handle: WalletHandle, export_config: ExportConfig) -> IndyResult<()> {
        trace!(
            "export > handle: {:?} export_config: {:?}",
            wallet_handle,
            secret!(&export_config)
        );

        let key_data =
            KeyDerivationData::from_passphrase_with_new_salt(&export_config.key, &export_config.key_derivation_method);

        let key = Self::_derive_key(&key_data).await?;

        let res = self
            .wallet_service
            .export_wallet(wallet_handle, &export_config, 0, (&key_data, &key))
            .await;

        trace!("export < {:?}", res);
        res
    }

    /// Creates a new secure wallet and then imports its content
    /// according to fields provided in import_config
    /// This can be seen as an indy_create_wallet call with additional content import
    ///
    /// #Params
    /// config: Wallet configuration json.
    /// {
    ///   "id": string, Identifier of the wallet.
    ///         Configured storage uses this identifier to lookup exact wallet data placement.
    ///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
    ///                  'Default' storage type allows to store wallet data in the local file.
    ///                  Custom storage types can be registered with indy_register_wallet_storage
    /// call.   "storage_config": optional<object>, Storage configuration json. Storage type
    /// defines set of supported keys.                     Can be optional if storage supports
    /// default configuration.                     For 'default' storage type configuration is:
    ///   {
    ///     "path": optional<string>, Path to the directory with wallet files.
    ///             Defaults to $HOME/.indy_client/wallet.
    ///             Wallet will be stored in the file {path}/{id}/sqlite.db
    ///   }
    /// }
    /// credentials: Wallet credentials json
    /// {
    ///   "key": string, Key or passphrase used for wallet key derivation.
    ///                  Look to key_derivation_method param for information about supported key
    /// derivation methods.   "storage_credentials": optional<object> Credentials for wallet
    /// storage. Storage type defines set of supported keys.                          Can be
    /// optional if storage supports default configuration.                          For
    /// 'default' storage type should be empty.   "key_derivation_method": optional<string>
    /// Algorithm to use for wallet key derivation:                             ARGON2I_MOD -
    /// derive secured wallet master key (used by default)                             
    /// ARGON2I_INT - derive secured wallet master key (less secured but faster)                
    /// RAW - raw wallet key master provided (skip derivation).                                
    /// RAW keys can be generated with indy_generate_wallet_key call }
    /// import_config: Import settings json.
    /// {
    ///   "path": <string>, path of the file that contains exported wallet content
    ///   "key": <string>, key used for export of the wallet
    /// }
    ///
    /// #Returns
    /// Error code
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub async fn import(
        &self,
        config: Config,
        credentials: Credentials,
        import_config: ExportConfig,
    ) -> IndyResult<()> {
        trace!(
            "import > config: {:?} credentials: {:?} import_config: {:?}",
            &config,
            secret!(&credentials),
            secret!(&import_config)
        );
        // TODO: try to refactor to avoid usage of continue methods

        let (wallet_handle, key_data, import_key_data) = self
            .wallet_service
            .import_wallet_prepare(&config, &credentials, &import_config)
            .await?;

        let import_key = Self::_derive_key(&import_key_data).await?;
        let key = Self::_derive_key(&key_data).await?;

        let res = self
            .wallet_service
            .import_wallet_continue(wallet_handle, &config, &credentials, (import_key, key))
            .await;

        trace!("import < {:?}", res);

        res
    }

    /// Generate wallet master key.
    /// Returned key is compatible with "RAW" key derivation method.
    /// It allows to avoid expensive key derivation for use cases when wallet keys can be stored in
    /// a secure enclave.
    ///
    /// #Params
    /// config: (optional) key configuration json.
    /// {
    ///   "seed": string, (optional) Seed that allows deterministic key creation (if not set random
    /// one will be created).                              Can be UTF-8, base64 or hex string.
    /// }
    ///
    /// #Returns
    /// err: Error code
    ///
    /// #Errors
    /// Common*
    /// Wallet*
    pub fn generate_key(&self, config: Option<KeyConfig>) -> IndyResult<String> {
        trace!("generate_key > config: {:?}", secret!(&config));

        let seed = config
            .as_ref()
            .and_then(|config| config.seed.as_ref().map(String::as_str));

        let key = match self.crypto_service.convert_seed(seed)? {
            Some(seed) => randombytes::randombytes_deterministic(
                chacha20poly1305_ietf::KEYBYTES,
                &randombytes::Seed::from_slice(&seed[..])?,
            ),
            None => randombytes::randombytes(chacha20poly1305_ietf::KEYBYTES),
        };

        let res = key[..].to_base58();

        trace!("generate_key < res: {:?}", res);
        Ok(res)
    }

    async fn _derive_key(key_data: &KeyDerivationData) -> IndyResult<MasterKey> {
        key_data.calc_master_key()
        // let res = spawn_blocking(move || key_data.calc_master_key()).await?;
        // Ok(res)
    }
}
