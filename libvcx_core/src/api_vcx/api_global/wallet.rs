use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use aries_vcx::{
    aries_vcx_core::{
        anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
        wallet,
        wallet::{
            base_wallet::BaseWallet,
            indy::{
                internal::{close_search_wallet, fetch_next_records_wallet, open_search_wallet},
                wallet::{close_wallet, create_indy_wallet, import, open_wallet},
                IndySdkWallet, IssuerConfig, RestoreWalletConfigs, WalletConfig,
            },
            structs_io::UnpackMessageOutput,
        },
        SearchHandle, WalletHandle,
    },
    global::settings::DEFAULT_LINK_SECRET_ALIAS,
    protocols::mediated_connection::pairwise_info::PairwiseInfo,
};

use crate::{
    api_vcx::api_global::profile::{
        get_main_anoncreds, get_main_ledger_write, get_main_wallet, try_get_main_wallet,
    },
    errors::{
        error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
        mapping_from_ariesvcx::map_ariesvcx_result,
        mapping_from_ariesvcxcore::map_ariesvcx_core_result,
    },
};

pub static GLOBAL_BASE_WALLET: RwLock<Option<Arc<IndySdkWallet>>> = RwLock::new(None);
pub static GLOBAL_BASE_ANONCREDS: RwLock<Option<Arc<IndyCredxAnonCreds>>> = RwLock::new(None);

pub fn get_main_wallet_handle() -> LibvcxResult<WalletHandle> {
    get_main_wallet().map(|wallet| wallet.get_wallet_handle())
}

pub async fn export_main_wallet(path: &str, backup_key: &str) -> LibvcxResult<()> {
    let wallet_handle = get_main_wallet_handle()?;
    map_ariesvcx_core_result(
        wallet::indy::wallet::export_wallet(wallet_handle, path, backup_key).await,
    )
}

fn build_component_base_wallet(wallet_handle: WalletHandle) -> Arc<IndySdkWallet> {
    Arc::new(IndySdkWallet::new(wallet_handle))
}

fn setup_global_wallet(wallet_handle: WalletHandle) -> LibvcxResult<()> {
    // new way
    let base_wallet_impl = build_component_base_wallet(wallet_handle);
    let mut b_wallet = GLOBAL_BASE_WALLET.write()?;
    *b_wallet = Some(base_wallet_impl.clone());
    // anoncreds
    let base_anoncreds_impl = Arc::new(IndyCredxAnonCreds);
    let mut b_anoncreds = GLOBAL_BASE_ANONCREDS.write()?;
    *b_anoncreds = Some(base_anoncreds_impl);
    Ok(())
}

pub fn setup_wallet(handle: WalletHandle) -> LibvcxResult<()> {
    setup_global_wallet(handle)
}

pub async fn open_as_main_wallet(wallet_config: &WalletConfig) -> LibvcxResult<WalletHandle> {
    let handle = wallet::indy::wallet::open_wallet(wallet_config).await?;
    setup_wallet(handle)?;
    Ok(handle)
}

pub async fn create_and_open_as_main_wallet(
    wallet_config: &WalletConfig,
) -> LibvcxResult<WalletHandle> {
    let handle = wallet::indy::wallet::create_and_open_wallet(wallet_config).await?;
    setup_wallet(handle)?;
    Ok(handle)
}

pub async fn close_main_wallet() -> LibvcxResult<()> {
    let wallet = try_get_main_wallet()?;
    match wallet {
        None => {
            warn!("Skipping wallet close, no global wallet component available.")
        }
        Some(wallet) => {
            wallet::indy::wallet::close_wallet(wallet.get_wallet_handle()).await?;
            let mut b_wallet = GLOBAL_BASE_WALLET.write()?;
            *b_wallet = None;
        }
    }
    Ok(())
}

pub async fn create_main_wallet(config: &WalletConfig) -> LibvcxResult<()> {
    let wallet_handle = create_and_open_as_main_wallet(config).await?;
    trace!("Created wallet with handle {:?}", wallet_handle);
    let wallet = get_main_wallet()?;

    // If MS is already in wallet then just continue
    get_main_anoncreds()?
        .prover_create_link_secret(wallet.as_ref(), DEFAULT_LINK_SECRET_ALIAS)
        .await
        .ok();

    close_main_wallet().await?;
    Ok(())
}

pub async fn key_for_local_did(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.key_for_local_did(did).await)
}

pub async fn wallet_sign(vk: &str, data_raw: &[u8]) -> LibvcxResult<Vec<u8>> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.sign(vk, data_raw).await)
}

pub async fn wallet_verify(vk: &str, msg: &[u8], signature: &[u8]) -> LibvcxResult<bool> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.verify(vk, msg, signature).await)
}

pub async fn replace_did_keys_start(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.replace_did_keys_start(did).await)
}

pub async fn rotate_verkey_apply(did: &str, temp_vk: &str) -> LibvcxResult<()> {
    map_ariesvcx_result(
        aries_vcx::common::keys::rotate_verkey_apply(
            get_main_wallet()?.as_ref(),
            get_main_ledger_write()?.as_ref(),
            did,
            temp_vk,
        )
        .await,
    )
}

pub async fn wallet_unpack_message(payload: &[u8]) -> LibvcxResult<UnpackMessageOutput> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.unpack_message(payload).await)
}

pub async fn wallet_create_and_store_did(seed: Option<&str>) -> LibvcxResult<PairwiseInfo> {
    let wallet = get_main_wallet()?;
    let (pw_did, pw_vk) = wallet.create_and_store_my_did(seed, None).await?;
    Ok(PairwiseInfo { pw_did, pw_vk })
}

pub async fn wallet_configure_issuer(enterprise_seed: &str) -> LibvcxResult<IssuerConfig> {
    // TODO - future - use profile wallet to stop indy dependency
    let wallet = get_main_wallet_handle()?;
    map_ariesvcx_core_result(
        wallet::indy::wallet::wallet_configure_issuer(wallet, enterprise_seed).await,
    )
}

pub async fn wallet_add_wallet_record(
    type_: &str,
    id: &str,
    value: &str,
    option: Option<&str>,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let tags: Option<HashMap<String, String>> = option.map(serde_json::from_str).transpose()?;
    map_ariesvcx_core_result(wallet.add_wallet_record(type_, id, value, tags).await)
}

pub async fn wallet_update_wallet_record_value(
    xtype: &str,
    id: &str,
    value: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.update_wallet_record_value(xtype, id, value).await)
}

pub async fn wallet_update_wallet_record_tags(
    xtype: &str,
    id: &str,
    tags_json: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let tags: HashMap<String, String> = serde_json::from_str(tags_json)?;
    map_ariesvcx_core_result(wallet.update_wallet_record_tags(xtype, id, tags).await)
}

pub async fn wallet_add_wallet_record_tags(
    xtype: &str,
    id: &str,
    tags_json: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let tags: HashMap<String, String> = serde_json::from_str(tags_json)?;
    map_ariesvcx_core_result(wallet.add_wallet_record_tags(xtype, id, tags).await)
}

pub async fn wallet_delete_wallet_record_tags(
    xtype: &str,
    id: &str,
    tags_json: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.delete_wallet_record_tags(xtype, id, tags_json).await)
}

pub async fn wallet_get_wallet_record(
    xtype: &str,
    id: &str,
    options: &str,
) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.get_wallet_record(xtype, id, options).await)
}

pub async fn wallet_delete_wallet_record(xtype: &str, id: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.delete_wallet_record(xtype, id).await)
}

pub async fn wallet_open_search_wallet(
    xtype: &str,
    query_json: &str,
    options_json: &str,
) -> LibvcxResult<SearchHandle> {
    // TODO - future - use profile wallet to stop binding to indy
    let wallet_handle = get_main_wallet_handle()?;
    map_ariesvcx_core_result(
        open_search_wallet(wallet_handle, xtype, query_json, options_json).await,
    )
}

pub async fn wallet_close_search_wallet(wallet_search_handle: SearchHandle) -> LibvcxResult<()> {
    map_ariesvcx_core_result(close_search_wallet(wallet_search_handle).await)
}

pub async fn wallet_fetch_next_records_wallet(
    wallet_search_handle: SearchHandle,
    count: usize,
) -> LibvcxResult<String> {
    // TODO - future - use profile wallet to stop binding to indy
    let wallet_handle = get_main_wallet_handle()?;
    map_ariesvcx_core_result(
        fetch_next_records_wallet(wallet_handle, wallet_search_handle, count).await,
    )
}

pub async fn wallet_import(config: &RestoreWalletConfigs) -> LibvcxResult<()> {
    map_ariesvcx_core_result(import(config).await)
}

pub async fn wallet_migrate(wallet_config: &WalletConfig) -> LibvcxResult<()> {
    let src_wallet_handle = get_main_wallet_handle()?;
    info!("Assuring target wallet exists.");
    create_indy_wallet(wallet_config).await?;
    info!("Opening target wallet.");
    let dest_wallet_handle = open_wallet(wallet_config).await?;
    info!("Target wallet is ready.");

    let migration_res = wallet_migrator::migrate_wallet(
        src_wallet_handle,
        dest_wallet_handle,
        wallet_migrator::vdrtools2credx::migrate_any_record,
    )
    .await;

    info!("Closing source and target wallets");
    close_wallet(src_wallet_handle).await.ok();
    close_wallet(dest_wallet_handle).await.ok();

    migration_res.map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::WalletMigrationFailed, e))
}

#[cfg(test)]
pub mod test_utils {
    use ::test_utils::devsetup::TempFile;
    use aries_vcx::{
        aries_vcx_core::wallet::indy::WalletConfig,
        global::settings::{DEFAULT_WALLET_BACKUP_KEY, DEFAULT_WALLET_KEY, WALLET_KDF_RAW},
    };
    use aries_vcx_core::wallet::base_wallet::BaseWallet;

    use crate::{
        api_vcx::api_global::{
            profile::get_main_wallet,
            wallet::{
                close_main_wallet, create_and_open_as_main_wallet, create_main_wallet,
                export_main_wallet, open_as_main_wallet,
            },
        },
        errors::error::LibvcxResult,
    };

    fn _record() -> (&'static str, &'static str, &'static str) {
        ("type1", "id1", "value1")
    }

    pub async fn _create_main_wallet_and_its_backup() -> (TempFile, String, WalletConfig) {
        let wallet_name = &format!("export_test_wallet_{}", uuid::Uuid::new_v4());

        let export_file = TempFile::prepare_path(wallet_name);

        let wallet_config = WalletConfig {
            wallet_name: wallet_name.into(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();
        let wallet = get_main_wallet().unwrap();
        wallet.create_and_store_my_did(None, None).await.unwrap();
        let (type_, id, value) = _record();
        wallet
            .add_wallet_record(type_, id, value, None)
            .await
            .unwrap();
        export_main_wallet(&export_file.path, DEFAULT_WALLET_BACKUP_KEY)
            .await
            .unwrap();

        close_main_wallet().await.unwrap();

        // todo: import and verify
        (export_file, wallet_name.to_string(), wallet_config)
    }

    pub async fn _create_wallet() -> LibvcxResult<WalletConfig> {
        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());
        let config_wallet: WalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))?;
        create_main_wallet(&config_wallet).await?;
        Ok(config_wallet)
    }

    pub async fn _create_and_open_wallet() -> LibvcxResult<WalletConfig> {
        let config_wallet = _create_wallet().await?;
        open_as_main_wallet(&config_wallet).await?;
        Ok(config_wallet)
    }
}

#[cfg(test)]
mod tests {
    use aries_vcx::{
        aries_vcx_core::wallet::indy::{
            wallet::delete_wallet, RestoreWalletConfigs, WalletConfig, WalletRecord,
        },
        global::settings::{DEFAULT_WALLET_BACKUP_KEY, DEFAULT_WALLET_KEY, WALLET_KDF_RAW},
    };
    use test_utils::devsetup::{SetupMocks, TempFile};

    use crate::{
        api_vcx::api_global::wallet::{
            close_main_wallet, create_and_open_as_main_wallet, create_main_wallet,
            export_main_wallet, open_as_main_wallet,
            test_utils::{_create_and_open_wallet, _create_main_wallet_and_its_backup},
            wallet_add_wallet_record, wallet_delete_wallet_record, wallet_get_wallet_record,
            wallet_import, wallet_update_wallet_record_value,
        },
        errors::error::{LibvcxErrorKind, LibvcxResult},
    };

    #[tokio::test]
    async fn test_wallet_migrate() {
        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());
        let config: WalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();

        create_and_open_as_main_wallet(&config).await.unwrap();

        let wallet_name = format!("test_migrate_wallet_{}", uuid::Uuid::new_v4());
        let new_config: WalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();

        super::wallet_migrate(&new_config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_create() {
        let _setup = SetupMocks::init();

        let wallet_name = format!("test_create_wallet_{}", uuid::Uuid::new_v4());
        let config: WalletConfig = serde_json::from_value(json!({
            "wallet_name": wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();

        create_main_wallet(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_record_add_with_tag() {
        _create_and_open_wallet().await.unwrap();

        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let value = "Record Value".to_string();
        let tags = r#"{"tagName1":"tag1","tagName2":"tag2"}"#.to_string();

        wallet_add_wallet_record(&xtype, &id, &value, Some(&tags))
            .await
            .unwrap();
        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_record_add_with_no_tag() {
        _create_and_open_wallet().await.unwrap();

        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let value = "Record Value".to_string();

        wallet_add_wallet_record(&xtype, &id, &value, None)
            .await
            .unwrap();
        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_record_add_fails_with_duplication_error() {
        _create_and_open_wallet().await.unwrap();

        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let value = "Record Value".to_string();

        wallet_add_wallet_record(&xtype, &id, &value, None)
            .await
            .unwrap();
        let err = wallet_add_wallet_record(&xtype, &id, &value, None)
            .await
            .unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::DuplicationWalletRecord);
        close_main_wallet().await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_record_get_fails_if_record_does_not_exist() {
        _create_and_open_wallet().await.unwrap();

        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let options = json!({
            "retrieveType": true,
            "retrieveValue": true,
            "retrieveTags": false
        })
        .to_string();
        let _err = wallet_get_wallet_record(&xtype, &id, &options)
            .await
            .unwrap_err();
        // copilot demo: example
        close_main_wallet().await.unwrap();
    }

    async fn _add_and_get_wallet_record() -> LibvcxResult<()> {
        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let value = "Record Value".to_string();
        let tags = r#"{"tagName1":"tag1","tagName2":"tag2"}"#.to_string();

        wallet_add_wallet_record(&xtype, &id, &value, Some(&tags)).await?;

        let options = json!({
            "retrieveType": true,
            "retrieveValue": true,
            "retrieveTags": true
        })
        .to_string();

        let record = wallet_get_wallet_record(&xtype, &id, &options).await?;
        let record: WalletRecord = serde_json::from_str(&record)?;
        assert_eq!(record.value.unwrap(), value);
        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_record_delete() {
        _create_and_open_wallet().await.unwrap();

        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let value = "Record Value".to_string();

        wallet_add_wallet_record(&xtype, &id, &value, None)
            .await
            .unwrap();
        wallet_delete_wallet_record(&xtype, &id).await.unwrap();
        let err = wallet_delete_wallet_record(&xtype, &id).await.unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::WalletRecordNotFound);
        let err = wallet_get_wallet_record(&xtype, &id, "{}")
            .await
            .unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::WalletRecordNotFound);
    }

    #[tokio::test]
    async fn test_wallet_export_import() {
        let _setup = SetupMocks::init();
        let wallet_name = uuid::Uuid::new_v4().to_string();
        let export_file = TempFile::prepare_path(&wallet_name);
        let wallet_config = WalletConfig {
            wallet_name,
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();
        let backup_key = DEFAULT_WALLET_BACKUP_KEY;
        export_main_wallet(&export_file.path.to_string(), backup_key)
            .await
            .unwrap();
        close_main_wallet().await.unwrap();
        delete_wallet(&wallet_config).await.unwrap();
        let import_config: RestoreWalletConfigs = serde_json::from_value(json!({
            "wallet_name": wallet_config.wallet_name.clone(),
            "wallet_key": wallet_config.wallet_key.clone(),
            "exported_wallet_path": export_file.path,
            "backup_key": backup_key,
            "wallet_key_derivation": WALLET_KDF_RAW
        }))
        .unwrap();
        wallet_import(&import_config).await.unwrap();
        delete_wallet(&wallet_config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_open_with_incorrect_key_fails() {
        let _setup = SetupMocks::init();
        let wallet_name = uuid::Uuid::new_v4().to_string();
        let _export_file = TempFile::prepare_path(&wallet_name);
        let mut wallet_config = WalletConfig {
            wallet_name,
            wallet_key: DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_and_open_as_main_wallet(&wallet_config)
            .await
            .unwrap();
        close_main_wallet().await.unwrap();
        wallet_config.wallet_key = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFA2cAA".to_string();
        let err = open_as_main_wallet(&wallet_config).await.unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::WalletAccessFailed);
    }

    #[tokio::test]
    async fn test_wallet_open_with_wrong_name_fails() {
        let _setup = SetupMocks::init();

        let wallet_config: WalletConfig = serde_json::from_value(json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW,
        }))
        .unwrap();

        assert_eq!(
            open_as_main_wallet(&wallet_config)
                .await
                .unwrap_err()
                .kind(),
            LibvcxErrorKind::WalletNotFound
        )
    }

    #[tokio::test]
    async fn test_wallet_open_of_imported_wallet_succeeds() {
        let _setup = SetupMocks::init();

        let (export_wallet_path, wallet_name, wallet_config) =
            _create_main_wallet_and_its_backup().await;

        delete_wallet(&wallet_config).await.unwrap();

        let import_config = RestoreWalletConfigs {
            wallet_name: wallet_name.clone(),
            wallet_key: DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: Some(WALLET_KDF_RAW.into()),
        };
        wallet_import(&import_config).await.unwrap();

        let wallet_config: WalletConfig = serde_json::from_value(json!({
            "wallet_name": &wallet_name,
            "wallet_key": DEFAULT_WALLET_KEY,
            "wallet_key_derivation": WALLET_KDF_RAW,
        }))
        .unwrap();

        open_as_main_wallet(&wallet_config).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_import_of_opened_wallet_fails() {
        let _setup = SetupMocks::init();

        let (export_wallet_path, wallet_name, wallet_config) =
            _create_main_wallet_and_its_backup().await;

        open_as_main_wallet(&wallet_config).await.unwrap();

        let import_config = RestoreWalletConfigs {
            wallet_name,
            wallet_key: DEFAULT_WALLET_KEY.into(),
            exported_wallet_path: export_wallet_path.path.clone(),
            backup_key: DEFAULT_WALLET_BACKUP_KEY.to_string(),
            wallet_key_derivation: None,
        };
        assert_eq!(
            wallet_import(&import_config).await.unwrap_err().kind(),
            LibvcxErrorKind::DuplicationWallet
        )
    }

    #[tokio::test]
    async fn test_wallet_record_update() {
        _create_and_open_wallet().await.unwrap();

        let xtype = "record_type".to_string();
        let id = "123".to_string();
        let value = "Record Value".to_string();
        let new_value = "New Record Value".to_string();

        let err = wallet_update_wallet_record_value(&xtype, &id, &new_value)
            .await
            .unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::WalletRecordNotFound);
        wallet_add_wallet_record(&xtype, &id, &value, None)
            .await
            .unwrap();
        wallet_update_wallet_record_value(&xtype, &id, &new_value)
            .await
            .unwrap();
        let record = wallet_get_wallet_record(&xtype, &id, "{}").await.unwrap();
        let record: WalletRecord = serde_json::from_str(&record).unwrap();
        assert_eq!(record.value.unwrap(), new_value);
    }
}
