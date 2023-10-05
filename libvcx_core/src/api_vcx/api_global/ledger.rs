use std::str::FromStr;

use aries_vcx::{
    aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions,
    common::ledger::{
        service_didsov::{DidSovServiceType, EndpointDidSov},
        transactions::{
            clear_attr, get_attr, get_service, write_endorser_did, write_endpoint,
            write_endpoint_legacy,
        },
    },
};
use aries_vcx_core::ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite};
use diddoc_legacy::aries::service::AriesService;
use url::Url;

use super::profile::{get_main_ledger_read, get_main_ledger_write, update_taa_configuration};
use crate::{
    api_vcx::api_global::profile::get_main_wallet,
    errors::{
        error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
        mapping_from_ariesvcx::map_ariesvcx_result,
        mapping_from_ariesvcxcore::map_ariesvcx_core_result,
    },
};

pub async fn endorse_transaction(transaction: &str, endorser_did: &str) -> LibvcxResult<()> {
    let ledger = get_main_ledger_write()?;
    map_ariesvcx_core_result(ledger.endorse_transaction(endorser_did, transaction).await)
}

pub async fn get_ledger_txn(seq_no: i32, submitter_did: Option<String>) -> LibvcxResult<String> {
    let ledger = get_main_ledger_read()?;
    map_ariesvcx_core_result(
        ledger
            .get_ledger_txn(seq_no, submitter_did.as_deref())
            .await,
    )
}

pub async fn rotate_verkey(did: &str) -> LibvcxResult<()> {
    let result = aries_vcx::common::keys::rotate_verkey(
        get_main_wallet()?.as_ref(),
        get_main_ledger_write()?.as_ref(),
        did,
    )
    .await;
    map_ariesvcx_result(result)
}

pub async fn get_verkey_from_ledger(did: &str) -> LibvcxResult<String> {
    let indy_ledger = get_main_ledger_read()?;
    map_ariesvcx_result(
        aries_vcx::common::keys::get_verkey_from_ledger(indy_ledger.as_ref(), did).await,
    )
}

pub async fn ledger_write_endpoint_legacy(
    target_did: &str,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<AriesService> {
    let service =
        AriesService::create()
            .set_service_endpoint(Url::from_str(&endpoint).map_err(|err| {
                LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string())
            })?)
            .set_recipient_keys(recipient_keys)
            .set_routing_keys(routing_keys);
    write_endpoint_legacy(get_main_ledger_write()?.as_ref(), target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_write_endpoint(
    target_did: &str,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<EndpointDidSov> {
    let service =
        EndpointDidSov::create()
            .set_service_endpoint(Url::from_str(&endpoint).map_err(|err| {
                LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string())
            })?)
            .set_types(Some(vec![
                DidSovServiceType::Endpoint,
                DidSovServiceType::DidCommunication,
            ]))
            .set_routing_keys(Some(routing_keys));
    write_endpoint(get_main_ledger_write()?.as_ref(), target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_get_service(target_did: &str) -> LibvcxResult<AriesService> {
    let target_did = target_did.to_owned();
    map_ariesvcx_result(get_service(get_main_ledger_read()?.as_ref(), &target_did).await)
}

pub async fn ledger_get_attr(target_did: &str, attr: &str) -> LibvcxResult<String> {
    map_ariesvcx_result(get_attr(get_main_ledger_read()?.as_ref(), target_did, attr).await)
}

pub async fn ledger_clear_attr(target_did: &str, attr: &str) -> LibvcxResult<String> {
    map_ariesvcx_result(clear_attr(get_main_ledger_write()?.as_ref(), target_did, attr).await)
}

pub async fn ledger_write_endorser_did(
    submitter_did: &str,
    target_did: &str,
    target_vk: &str,
    alias: Option<String>,
) -> LibvcxResult<String> {
    map_ariesvcx_result(
        write_endorser_did(
            get_main_ledger_write()?.as_ref(),
            submitter_did,
            target_did,
            target_vk,
            alias,
        )
        .await,
    )
}

pub async fn ledger_get_txn_author_agreement() -> LibvcxResult<String> {
    get_main_ledger_read()?
        .as_ref()
        .get_txn_author_agreement()
        .await?
        .ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::LedgerItemNotFound,
                "No transaction author agreement found",
            )
        })
}

pub fn set_taa_configuration(
    text: String,
    version: String,
    acceptance_mechanism: String,
) -> LibvcxResult<()> {
    let taa_options = TxnAuthrAgrmtOptions {
        text,
        version,
        mechanism: acceptance_mechanism,
    };
    update_taa_configuration(taa_options)
}

pub fn get_taa_configuration() -> LibvcxResult<Option<TxnAuthrAgrmtOptions>> {
    super::profile::get_taa_configuration()
}

#[cfg(test)]
pub mod tests {
    use aries_vcx::{
        aries_vcx_core::ledger::indy::pool::test_utils::{
            create_genesis_txn_file, create_testpool_genesis_txn_file, get_temp_file_path,
            get_txns_sovrin_testnet,
        },
        global::settings::DEFAULT_GENESIS_PATH,
        utils::devsetup::SetupEmpty,
    };

    use crate::api_vcx::api_global::{
        ledger::{get_taa_configuration, ledger_get_txn_author_agreement, set_taa_configuration},
        pool::{open_main_pool, LibvcxLedgerConfig},
        wallet::test_utils::_create_and_open_wallet,
    };

    #[tokio::test]
    async fn test_vcx_get_sovrin_taa() {
        let _setup = SetupEmpty::init();
        _create_and_open_wallet().await.unwrap();
        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH)
            .to_str()
            .unwrap()
            .to_string();
        create_genesis_txn_file(&genesis_path, Box::new(get_txns_sovrin_testnet));
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
            exclude_nodes: Some(vec!["NECValidator".into(), "Entrustient".into()]),
        };
        open_main_pool(&config).await.unwrap();

        let taa = ledger_get_txn_author_agreement().await.unwrap();
        let taa_parsed = serde_json::from_str::<serde_json::Value>(&taa).unwrap();
        assert!(taa_parsed["text"].is_string());
        assert!(taa_parsed["version"].is_string());
        assert!(taa_parsed["digest"].is_string());
    }

    #[tokio::test]
    async fn test_vcx_set_active_txn_author_agreement_meta() {
        let _setup = SetupEmpty::init();
        _create_and_open_wallet().await.unwrap();
        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH)
            .to_str()
            .unwrap()
            .to_string();
        create_testpool_genesis_txn_file(&genesis_path);
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
            exclude_nodes: None,
        };
        open_main_pool(&config).await.unwrap();

        assert!(get_taa_configuration().unwrap().is_none());

        let text = "text";
        let version = "1.0.0";
        let acc_mech_type = "type 1";

        set_taa_configuration(text.into(), version.into(), acc_mech_type.into()).unwrap();
        let auth_agreement = get_taa_configuration().unwrap().unwrap();

        let expected = json!({
            "text": text,
            "version": version,
            "mechanism": acc_mech_type
        });

        let auth_agreement = serde_json::to_value(auth_agreement).unwrap();
        assert_eq!(expected, auth_agreement);
    }
}
