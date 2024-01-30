#[cfg(feature = "askar_wallet")]
pub mod askar;
#[cfg(feature = "askar_wallet")]
pub use askar as wallet;

#[cfg(feature = "vdrtools_wallet")]
pub mod indy;
#[cfg(feature = "vdrtools_wallet")]
pub use indy as wallet;

pub static GLOBAL_BASE_ANONCREDS: RwLock<Option<Arc<IndyCredxAnonCreds>>> = RwLock::new(None);

pub fn setup_global_anoncreds() -> LibvcxResult<()> {
    let base_anoncreds_impl = Arc::new(IndyCredxAnonCreds);
    let mut b_anoncreds = GLOBAL_BASE_ANONCREDS.write()?;
    *b_anoncreds = Some(base_anoncreds_impl);
    Ok(())
}

pub async fn export_main_wallet(path: &str, backup_key: &str) -> LibvcxResult<()> {
    let main_wallet = get_main_wallet()?;
    map_ariesvcx_core_result(main_wallet.as_ref().export_wallet(path, backup_key).await)
}

use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use aries_vcx::{
    aries_vcx_core::{
        anoncreds::credx_anoncreds::IndyCredxAnonCreds, wallet::structs_io::UnpackMessageOutput,
    },
    protocols::mediated_connection::pairwise_info::PairwiseInfo,
};
use aries_vcx_core::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind},
    wallet::{
        base_wallet::{
            did_wallet::DidWallet, issuer_config::IssuerConfig, record::Record,
            record_category::RecordCategory, record_wallet::RecordWallet,
            search_filter::SearchFilter, BaseWallet,
        },
        indy::indy_wallet_record::IndyWalletRecord,
        record_tags::RecordTags,
    },
};
use public_key::{Key, KeyType};

use crate::{
    api_vcx::api_global::profile::{get_main_ledger_write, get_main_wallet},
    errors::{
        error::LibvcxResult, mapping_from_ariesvcx::map_ariesvcx_result,
        mapping_from_ariesvcxcore::map_ariesvcx_core_result,
    },
};

#[cfg(all(feature = "vdrtools_wallet", feature = "askar_wallet"))]
compile_error!("features `vdrtools_wallet` and `askar_wallet` are mutually exclusive");

pub async fn key_for_local_did(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;

    map_ariesvcx_core_result(wallet.key_for_did(did).await.map(|key| key.base58()))
}

pub async fn wallet_sign(vk: &str, data_raw: &[u8]) -> LibvcxResult<Vec<u8>> {
    let wallet = get_main_wallet()?;

    let verkey = Key::from_base58(vk, KeyType::Ed25519)?;
    map_ariesvcx_core_result(wallet.sign(&verkey, data_raw).await)
}

pub async fn wallet_verify(vk: &str, msg: &[u8], signature: &[u8]) -> LibvcxResult<bool> {
    let wallet = get_main_wallet()?;

    let verkey = Key::from_base58(vk, KeyType::Ed25519)?;
    map_ariesvcx_core_result(wallet.verify(&verkey, msg, signature).await)
}

pub async fn replace_did_keys_start(did: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;

    map_ariesvcx_core_result(
        wallet
            .replace_did_key_start(did, None)
            .await
            .map(|key| key.base58()),
    )
}

pub async fn rotate_verkey_apply(did: &str, temp_vk: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_result(
        aries_vcx::common::keys::rotate_verkey_apply(
            wallet.as_ref(),
            get_main_ledger_write()?.as_ref(),
            &did.parse()?,
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
    let did_data = wallet.create_and_store_my_did(seed, None).await?;
    Ok(PairwiseInfo {
        pw_did: did_data.did().into(),
        pw_vk: did_data.verkey().base58(),
    })
}

pub async fn wallet_configure_issuer(enterprise_seed: &str) -> LibvcxResult<IssuerConfig> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(wallet.configure_issuer(enterprise_seed).await)
}

pub async fn wallet_add_wallet_record(
    type_: &str,
    id: &str,
    value: &str,
    option: Option<&str>,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let tags: Option<RecordTags> = option.map(serde_json::from_str).transpose()?;

    let record = if let Some(record_tags) = tags {
        Record::builder()
            .name(id.into())
            .category(RecordCategory::from_str(type_)?)
            .value(value.into())
            .tags(record_tags)
            .build()
    } else {
        Record::builder()
            .name(id.into())
            .category(RecordCategory::from_str(type_)?)
            .value(value.into())
            .build()
    };

    map_ariesvcx_core_result(wallet.add_record(record).await)
}

pub async fn wallet_update_wallet_record_value(
    xtype: &str,
    id: &str,
    value: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(
        wallet
            .update_record_value(RecordCategory::from_str(xtype)?, id, value)
            .await,
    )
}

pub async fn wallet_update_wallet_record_tags(
    xtype: &str,
    id: &str,
    tags_json: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let tags: RecordTags = serde_json::from_str(tags_json)?;
    map_ariesvcx_core_result(
        wallet
            .update_record_tags(RecordCategory::from_str(xtype)?, id, tags)
            .await,
    )
}

pub async fn wallet_add_wallet_record_tags(
    xtype: &str,
    id: &str,
    tags_json: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let record = wallet
        .get_record(RecordCategory::from_str(xtype)?, id)
        .await?;

    let tags = {
        let mut tags: RecordTags = serde_json::from_str(tags_json)?;
        tags.merge(record.tags().clone());
        tags
    };

    map_ariesvcx_core_result(
        wallet
            .update_record_tags(RecordCategory::from_str(xtype)?, id, tags)
            .await,
    )
}

pub async fn wallet_delete_wallet_record_tags(
    xtype: &str,
    id: &str,
    tags_json: &str,
) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    let tags: RecordTags = serde_json::from_str(tags_json)?;

    let record = wallet
        .get_record(RecordCategory::from_str(xtype)?, id)
        .await?;

    let mut found_tags = record.tags().clone();
    for key in tags {
        found_tags.remove(key);
    }

    map_ariesvcx_core_result(
        wallet
            .update_record_tags(RecordCategory::from_str(xtype)?, id, found_tags)
            .await,
    )
}

pub async fn wallet_get_wallet_record(
    xtype: &str,
    id: &str,
    _options: &str,
) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;

    map_ariesvcx_result(
        wallet
            .get_record(RecordCategory::from_str(xtype)?, id)
            .await
            .map(|res| {
                let wallet_record = IndyWalletRecord::from_record(res)?;

                Ok(serde_json::to_string(&wallet_record)?)
            })?,
    )
}

pub async fn wallet_delete_wallet_record(xtype: &str, id: &str) -> LibvcxResult<()> {
    let wallet = get_main_wallet()?;
    map_ariesvcx_core_result(
        wallet
            .delete_record(RecordCategory::from_str(xtype)?, id)
            .await,
    )
}

pub async fn wallet_search_records(xtype: &str, query_json: &str) -> LibvcxResult<String> {
    let wallet = get_main_wallet()?;
    let records = wallet
        .search_record(
            RecordCategory::from_str(xtype)?,
            Some(SearchFilter::JsonFilter(query_json.into())),
        )
        .await?;

    let indy_records = records
        .into_iter()
        .map(IndyWalletRecord::from_record)
        .collect::<Result<Vec<_>, _>>()?;

    let res = serde_json::to_string(&indy_records)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, err));

    map_ariesvcx_core_result(res)
}

#[cfg(test)]
pub mod test_utils {
    use ::test_utils::devsetup::TempFile;
    use aries_vcx::global::settings::DEFAULT_WALLET_BACKUP_KEY;
    use aries_vcx_core::wallet::base_wallet::{
        record::Record, record_category::RecordCategory, BaseWallet,
    };

    use crate::api_vcx::api_global::wallet::{export_main_wallet, wallet::close_main_wallet};

    pub async fn setup_wallet_backup(wallet: &impl BaseWallet, export_file: &TempFile) {
        wallet.create_and_store_my_did(None, None).await.unwrap();

        let new_record = Record::builder()
            .name("id1".to_owned())
            .category(RecordCategory::default())
            .value("value1".to_owned())
            .build();

        wallet.add_record(new_record).await.unwrap();
        export_main_wallet(&export_file.path, DEFAULT_WALLET_BACKUP_KEY)
            .await
            .unwrap();

        close_main_wallet().await.unwrap();
    }
}

// TODO: remove feature flag when closing wallet is implemented for askar
#[cfg(feature = "vdrtools_wallet")]
#[cfg(test)]
mod tests {
    use aries_vcx_core::wallet::{
        base_wallet::record_category::RecordCategory, indy::indy_wallet_record::IndyWalletRecord,
    };

    use crate::{
        api_vcx::api_global::wallet::{
            wallet::{close_main_wallet, test_utils::_create_and_open_wallet},
            wallet_add_wallet_record, wallet_delete_wallet_record, wallet_get_wallet_record,
            wallet_update_wallet_record_value,
        },
        errors::error::{LibvcxErrorKind, LibvcxResult},
    };

    #[tokio::test]
    async fn test_wallet_record_add_with_tag() {
        _create_and_open_wallet().await.unwrap();

        let xtype = RecordCategory::default().to_string();
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

        let xtype = RecordCategory::default().to_string();
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

        let xtype = RecordCategory::default().to_string();
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

        let xtype = RecordCategory::default().to_string();
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
        let xtype = RecordCategory::default().to_string();
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
        let record: IndyWalletRecord = serde_json::from_str(&record)?;
        assert_eq!(record.value.unwrap(), value);
        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_record_delete() {
        _create_and_open_wallet().await.unwrap();

        let xtype = RecordCategory::default().to_string();
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
    async fn test_wallet_record_update() {
        _create_and_open_wallet().await.unwrap();

        let xtype = RecordCategory::default().to_string();
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
        let record: IndyWalletRecord = serde_json::from_str(&record).unwrap();
        assert_eq!(record.value.unwrap(), new_value);
    }
}
