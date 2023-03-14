use aries_vcx::common::ledger::service_didsov::{DidSovServiceType, EndpointDidSov};
use aries_vcx::common::ledger::transactions::{
    clear_attr, get_attr, get_service, write_endpoint, write_endpoint_legacy,
};
use aries_vcx::global::settings::CONFIG_INSTITUTION_DID;
use aries_vcx::messages::diddoc::aries::service::AriesService;
use aries_vcx::messages::protocols::connection::did::Did;

use crate::{
    api_vcx::api_global::{profile::get_main_profile, settings::get_config_value},
    errors::{error::LibvcxResult, mapping_from_ariesvcx::map_ariesvcx_result},
};

pub async fn endorse_transaction(transaction: &str) -> LibvcxResult<()> {
    let endorser_did = get_config_value(CONFIG_INSTITUTION_DID)?;

    let profile = get_main_profile()?;
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.endorse_transaction(&endorser_did, transaction).await)
}

pub async fn get_ledger_txn(seq_no: i32, submitter_did: Option<String>) -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.get_ledger_txn(seq_no, submitter_did.as_deref()).await)
}

pub async fn rotate_verkey(did: &str) -> LibvcxResult<()> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(aries_vcx::common::keys::rotate_verkey(&profile, did).await)
}

pub async fn get_verkey_from_ledger(did: &str) -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(aries_vcx::common::keys::get_verkey_from_ledger(&profile, did).await)
}

pub async fn ledger_write_endpoint_legacy(
    target_did: &str,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<AriesService> {
    let service = AriesService::create()
        .set_service_endpoint(endpoint)
        .set_recipient_keys(recipient_keys)
        .set_routing_keys(routing_keys);
    let profile = get_main_profile()?;
    write_endpoint_legacy(&profile, target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_write_endpoint(
    target_did: &str,
    routing_keys: Vec<String>,
    endpoint: String,
) -> LibvcxResult<EndpointDidSov> {
    let service = EndpointDidSov::create()
        .set_service_endpoint(endpoint)
        .set_types(Some(vec![
            DidSovServiceType::Endpoint,
            DidSovServiceType::DidCommunication,
        ]))
        .set_routing_keys(Some(routing_keys));
    let profile = get_main_profile()?;
    write_endpoint(&profile, target_did, &service).await?;
    Ok(service)
}

pub async fn ledger_get_service(target_did: &str) -> LibvcxResult<AriesService> {
    let target_did = Did::new(target_did)?;
    let profile = get_main_profile()?;
    map_ariesvcx_result(get_service(&profile, &target_did).await)
}

pub async fn ledger_get_attr(target_did: &str, attr: &str) -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(get_attr(&profile, &target_did, attr).await)
}

pub async fn ledger_clear_attr(target_did: &str, attr: &str) -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    map_ariesvcx_result(clear_attr(&profile, &target_did, attr).await)
}

pub async fn ledger_get_txn_author_agreement() -> LibvcxResult<String> {
    let profile = get_main_profile()?;
    let ledger = profile.inject_ledger();
    map_ariesvcx_result(ledger.get_txn_author_agreement().await)
}

pub fn ledger_set_txn_author_agreement(
    text: Option<String>,
    version: Option<String>,
    hash: Option<String>,
    acc_mech_type: String,
    time_of_acceptance: u64,
) -> LibvcxResult<()> {
    map_ariesvcx_result(aries_vcx::utils::author_agreement::set_txn_author_agreement(
        text,
        version,
        hash,
        acc_mech_type,
        time_of_acceptance,
    ))
}

#[cfg(test)]
pub mod tests {
    use crate::api_vcx::api_global::ledger::{ledger_get_txn_author_agreement, ledger_set_txn_author_agreement};
    use crate::api_vcx::api_global::settings::get_config_value;
    use aries_vcx::global::settings::{set_test_configs, CONFIG_TXN_AUTHOR_AGREEMENT};
    use aries_vcx::utils::devsetup::SetupMocks;

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_set_active_txn_author_agreement_meta() {
        let _setup = SetupMocks::init();

        assert!(&get_config_value(CONFIG_TXN_AUTHOR_AGREEMENT).is_err());

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
        let auth_agreement = get_config_value(CONFIG_TXN_AUTHOR_AGREEMENT).unwrap();

        let expected = json!({
            "text": text,
            "version": version,
            "acceptanceMechanismType": acc_mech_type,
            "timeOfAcceptance": time_of_acceptance,
        });

        let auth_agreement = serde_json::from_str::<::serde_json::Value>(&auth_agreement).unwrap();

        assert_eq!(expected, auth_agreement);

        // todo: delete the reset below?
        set_test_configs();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_vcx_get_ledger_author_agreement() {
        let _setup = SetupMocks::init();

        let agreement = ledger_get_txn_author_agreement().await.unwrap();
        assert_eq!(aries_vcx::utils::constants::DEFAULT_AUTHOR_AGREEMENT, agreement);
    }
}
