use std::str::FromStr;

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

pub fn ledger_set_txn_author_agreement(
    text: Option<String>,
    version: Option<String>,
    hash: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u64,
) -> LibvcxResult<()> {
    map_ariesvcx_result(
        aries_vcx::aries_vcx_core::global::author_agreement::set_vdrtools_config_txn_author_agreement(
            text,
            version,
            hash,
            acc_mech_type,
            time_of_acceptance,
        )
        .map_err(|err| err.into()),
    )
}

#[cfg(test)]
pub mod tests {
    use crate::api_vcx::api_global::ledger::{ledger_get_txn_author_agreement, ledger_set_txn_author_agreement};
    use crate::api_vcx::api_global::settings::get_config_value;
    use aries_vcx::aries_vcx_core::global::author_agreement::get_vdrtools_config_txn_author_agreement;
    use aries_vcx::global::settings::CONFIG_TXN_AUTHOR_AGREEMENT;
    use aries_vcx::utils::devsetup::SetupMocks;

    #[tokio::test]
    async fn test_vcx_set_active_txn_author_agreement_meta() {
        let _setup = SetupMocks::init();

        assert!(&get_vdrtools_config_txn_author_agreement().unwrap().is_none());

        let text = "text";
        let version = "1.0.0";
        let acc_mech_type = "type 1";
        let time_of_acceptance = 123456789;

        ledger_set_txn_author_agreement(
            Some(text.into()),
            Some(version.into()),
            None,
            acc_mech_type.into(),
            time_of_acceptance,
        )
        .unwrap();
        let auth_agreement = get_vdrtools_config_txn_author_agreement().unwrap().unwrap();

        let expected = json!({
            "text": text,
            "version": version,
            "acceptanceMechanismType": acc_mech_type,
            "timeOfAcceptance": time_of_acceptance,
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
