use std::str::FromStr;

use aries_vcx::aries_vcx_core::ledger::base_ledger::TxnAuthrAgrmtOptions;
use aries_vcx::common::ledger::service_didsov::{DidSovServiceType, EndpointDidSov};
use aries_vcx::common::ledger::transactions::{
    clear_attr, get_attr, get_service, write_endpoint, write_endpoint_legacy,
};
use aries_vcx::global::settings::CONFIG_INSTITUTION_DID;
use diddoc_legacy::aries::service::AriesService;
use url::Url;

use crate::api_vcx::api_global::profile::{
    get_main_indy_ledger_read, get_main_indy_ledger_write, get_main_profile, get_main_wallet,
};
use crate::api_vcx::api_global::settings::get_config_value;
use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use crate::errors::mapping_from_ariesvcx::map_ariesvcx_result;
use crate::errors::mapping_from_ariesvcxcore::map_ariesvcx_core_result;

pub async fn endorse_transaction(transaction: &str) -> LibvcxResult<()> {
    let endorser_did = get_config_value(CONFIG_INSTITUTION_DID)?;

    let ledger = get_main_indy_ledger_write()?;
    map_ariesvcx_core_result(ledger.endorse_transaction(&endorser_did, transaction).await)
}

pub async fn get_ledger_txn(seq_no: i32, submitter_did: Option<String>) -> LibvcxResult<String> {
    let ledger = get_main_indy_ledger_read()?;
    map_ariesvcx_core_result(ledger.get_ledger_txn(seq_no, submitter_did.as_deref()).await)
}

pub async fn rotate_verkey(did: &str) -> LibvcxResult<()> {
    let result = aries_vcx::common::keys::rotate_verkey(&get_main_wallet()?, &get_main_indy_ledger_write()?, did).await;
    map_ariesvcx_result(result)
}

pub async fn get_verkey_from_ledger(did: &str) -> LibvcxResult<String> {
    let indy_ledger = get_main_indy_ledger_read()?;
    map_ariesvcx_result(aries_vcx::common::keys::get_verkey_from_ledger(&indy_ledger, did).await)
}

pub async fn ledger_write_endpoint_legacy(
    target_did: &str,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<AriesService> {
    let service = AriesService::create()
        .set_service_endpoint(
            Url::from_str(&endpoint)
                .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string()))?,
        )
        .set_recipient_keys(recipient_keys)
        .set_routing_keys(routing_keys);
    write_endpoint_legacy(&get_main_indy_ledger_write()?, target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_write_endpoint(
    target_did: &str,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<EndpointDidSov> {
    let service = EndpointDidSov::create()
        .set_service_endpoint(
            Url::from_str(&endpoint)
                .map_err(|err| LibvcxError::from_msg(LibvcxErrorKind::InvalidUrl, err.to_string()))?,
        )
        .set_types(Some(vec![
            DidSovServiceType::Endpoint,
            DidSovServiceType::DidCommunication,
        ]))
        .set_routing_keys(Some(routing_keys));
    write_endpoint(&get_main_indy_ledger_write()?, target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_get_service(target_did: &str) -> LibvcxResult<AriesService> {
    let target_did = target_did.to_owned();
    map_ariesvcx_result(get_service(&get_main_indy_ledger_read()?, &target_did).await)
}

pub async fn ledger_get_attr(target_did: &str, attr: &str) -> LibvcxResult<String> {
    map_ariesvcx_result(get_attr(&get_main_indy_ledger_read()?, &target_did, attr).await)
}

pub async fn ledger_clear_attr(target_did: &str, attr: &str) -> LibvcxResult<String> {
    map_ariesvcx_result(clear_attr(&get_main_indy_ledger_write()?, &target_did, attr).await)
}

pub async fn ledger_get_txn_author_agreement() -> LibvcxResult<String> {
    get_main_indy_ledger_read()?
        .get_txn_author_agreement()
        .await?
        .ok_or_else(|| {
            LibvcxError::from_msg(
                LibvcxErrorKind::LedgerItemNotFound,
                "No transaction author agreement found",
            )
        })
}

pub fn set_taa_configuration(text: String, version: String, acceptance_mechanism: String) -> LibvcxResult<()> {
    let taa_options = TxnAuthrAgrmtOptions {
        text,
        version,
        mechanism: acceptance_mechanism,
    };
    get_main_profile().update_taa_configuration(taa_options)
}

pub fn get_taa_configuration() -> LibvcxResult<Option<TxnAuthrAgrmtOptions>> {
    get_main_profile().get_taa_configuration()
}

#[cfg(test)]
pub mod tests {
    use crate::api_vcx::api_global::ledger::{
        get_taa_configuration, ledger_get_txn_author_agreement, set_taa_configuration,
    };
    use crate::api_vcx::api_global::pool::{open_main_pool, LibvcxLedgerConfig};
    use crate::api_vcx::api_global::wallet::test_utils::_create_and_open_wallet;
    use aries_vcx::aries_vcx_core::ledger::indy::pool::test_utils::{
        create_genesis_txn_file, create_testpool_genesis_txn_file, get_temp_file_path, get_txns_sovrin_testnet,
    };
    use aries_vcx::global::settings::DEFAULT_GENESIS_PATH;
    use aries_vcx::utils::devsetup::{SetupEmpty, SetupMocks};

    #[tokio::test]
    async fn test_vcx_get_sovrin_taa() {
        let _setup = SetupEmpty::init();
        _create_and_open_wallet().await.unwrap();
        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        create_genesis_txn_file(&genesis_path, Box::new(get_txns_sovrin_testnet));
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
            exclude_nodes: vec![],
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
        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_path);
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
            exclude_nodes: vec![],
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

        let auth_agreement = serde_json::to_value(&auth_agreement).unwrap();
        assert_eq!(expected, auth_agreement);
    }

    #[tokio::test]
    async fn test_vcx_get_ledger_author_agreement() {
        let _setup = SetupMocks::init();

        let agreement = ledger_get_txn_author_agreement().await.unwrap();
        assert_eq!(aries_vcx::utils::constants::DEFAULT_AUTHOR_AGREEMENT, agreement);
    }
}
