#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "vcx"]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]
extern crate base64;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate indy_sys;
extern crate indyrs as indy;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate time;
extern crate url;
extern crate uuid;

#[macro_use]
pub mod utils;
pub mod settings;
#[macro_use]
pub mod messages;

pub mod api;
pub mod connection;
pub mod issuer_credential;
pub mod credential_request;
pub mod proof;
pub mod schema;
pub mod credential_def;
pub mod error;
pub mod credential;
pub mod disclosed_proof;

pub mod aries;
mod proof_utils;
mod disclosed_proof_utils;
mod filters;

#[allow(unused_imports)]
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use rand::Rng;
    use serde_json::Value;

    use api::ProofStateType;
    use api::VcxStateType;
    use connection;
    use credential;
    use disclosed_proof;
    use issuer_credential;
    use proof;
    use settings;
    use filters;
    use utils::{
        constants::TEST_TAILS_FILE,
        devsetup::{set_consumer, set_institution},
        get_temp_dir_path,
    };
    use utils::devsetup::*;

    use super::*;

    #[cfg(feature = "agency_pool_tests")]
    #[cfg(feature = "to_restore")] // message type spec/pairwise/1.0/UPDATE_CONN_STATUS no implemented in nodevcx agency
    #[test]
    fn test_delete_connection() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();

        let alice = connection::create_connection("alice").unwrap();
        connection::connect(alice).unwrap();
        connection::delete_connection(alice).unwrap();
        assert!(connection::release(alice).is_err());
    }

    fn attr_names() -> (String, String, String, String, String) {
        let address1 = "Address1".to_string();
        let address2 = "address2".to_string();
        let city = "CITY".to_string();
        let state = "State".to_string();
        let zip = "zip".to_string();
        (address1, address2, city, state, zip)
    }

    fn requested_attrs(did: &str, schema_id: &str, cred_def_id: &str, from: Option<u64>, to: Option<u64>) -> Value {
        let (address1, address2, city, state, zip) = attr_names();
        json!([
           {
              "name":address1,
               "non_revoked": {"from": from, "to": to},
              "restrictions": [{
                "issuer_did": did,
                "schema_id": schema_id,
                "cred_def_id": cred_def_id,
               }]
           },
           {
              "name":address2,
               "non_revoked": {"from": from, "to": to},
              "restrictions": [{
                "issuer_did": did,
                "schema_id": schema_id,
                "cred_def_id": cred_def_id,
               }],
           },
           {
              "name":city,
               "non_revoked": {"from": from, "to": to},
              "restrictions": [{
                "issuer_did": did,
                "schema_id": schema_id,
                "cred_def_id": cred_def_id,
               }]
           },
           {
              "name":state,
               "non_revoked": {"from": from, "to": to},
              "restrictions": [{
                "issuer_did": did,
                "schema_id": schema_id,
                "cred_def_id": cred_def_id,
               }]
           },
           {
              "name":zip,
               "non_revoked": {"from": from, "to": to},
              "restrictions": [{
                "issuer_did": did,
                "schema_id": schema_id,
                "cred_def_id": cred_def_id,
               }]
           }
        ])
    }

    fn create_and_send_cred_offer(did: &str, cred_def_handle: u32, connection: u32, credential_data: &str) -> u32 {
        set_institution(None);
        info!("create_and_send_cred_offer >> creating issuer credential");
        let handle_cred = issuer_credential::issuer_credential_create(cred_def_handle,
                                                                      "1".to_string(),
                                                                      did.to_string(),
                                                                      "credential_name".to_string(),
                                                                      credential_data.to_string(),
                                                                      1).unwrap();
        info!("create_and_send_cred_offer :: sending credential offer");
        issuer_credential::send_credential_offer(handle_cred, connection).unwrap();
        info!("create_and_send_cred_offer :: credential offer was sent");
        thread::sleep(Duration::from_millis(2000));
        handle_cred
    }

    fn send_cred_req(connection: u32, consumer_handle: Option<u32>) -> u32 {
        info!("send_cred_req >>> switching to consumer");
        set_consumer(consumer_handle);
        info!("send_cred_req :: getting offers");
        let credential_offers = credential::get_credential_offer_messages(connection).unwrap();
        let offers: Value = serde_json::from_str(&credential_offers).unwrap();
        let offers = serde_json::to_string(&offers[0]).unwrap();
        info!("send_cred_req :: creating credential from offer");
        let credential = credential::credential_create_with_offer("TEST_CREDENTIAL", &offers).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, credential::get_state(credential).unwrap());
        info!("send_cred_req :: sending credential request");
        credential::send_credential_request(credential, connection).unwrap();
        thread::sleep(Duration::from_millis(2000));
        credential
    }

    fn send_credential(issuer_handle: u32, connection: u32, credential_handle: u32, consumer_handle: Option<u32>) {
        info!("send_credential >>> switching to institution");
        set_institution(None);
        info!("send_credential >>> getting offers");
        issuer_credential::update_state(issuer_handle, None, None).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, issuer_credential::get_state(issuer_handle).unwrap());
        info!("sending credential");
        issuer_credential::send_credential(issuer_handle, connection).unwrap();
        thread::sleep(Duration::from_millis(2000));
        // AS CONSUMER STORE CREDENTIAL
        ::utils::devsetup::set_consumer(consumer_handle);
        credential::update_state(credential_handle, None, None).unwrap();
        thread::sleep(Duration::from_millis(2000));
        info!("storing credential");
        assert_eq!(VcxStateType::VcxStateAccepted as u32, credential::get_state(credential_handle).unwrap());
    }

    fn send_proof_request(connection_handle: u32, requested_attrs: &str, requested_preds: &str, revocation_interval: &str, institution_handle: Option<u32>, request_name: Option<&str>) -> u32 {
        set_institution(institution_handle);
        let proof_req_handle = proof::create_proof("1".to_string(),
                                                   requested_attrs.to_string(),
                                                   requested_preds.to_string(),
                                                   revocation_interval.to_string(),
                                                   String::from(request_name.unwrap_or("name"))).unwrap();
        proof::send_proof_request(proof_req_handle, connection_handle).unwrap();
        thread::sleep(Duration::from_millis(2000));
        proof_req_handle
    }

    fn create_proof(connection_handle: u32, consumer_handle: Option<u32>, request_name: Option<&str>) -> u32 {
        set_consumer(consumer_handle);
        info!("create_proof >>> getting proof request messages");
        let requests = {
            let _requests = disclosed_proof::get_proof_request_messages(connection_handle).unwrap();
            info!("create_proof :: get proof request messages returned {}", _requests);
            match request_name {
                Some(request_name) => {
                    let filtered = filters::filter_proof_requests_by_name(&_requests, request_name).unwrap();
                    info!("create_proof :: proof request messages filtered by name {}: {}", request_name, filtered);
                    filtered
                }
                _ => _requests
            }
        };
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = requests.as_array().unwrap();
        assert_eq!(requests.len(), 1);
        let request = serde_json::to_string(&requests[0]).unwrap();
        disclosed_proof::create_proof(::utils::constants::DEFAULT_PROOF_NAME, &request).unwrap()
    }

    fn generate_and_send_proof(proof_handle: u32, connection_handle: u32, selected_credentials: &str, consumer_handle: Option<u32>) {
        set_consumer(consumer_handle);
        info!("generate_and_send_proof >>> generating proof using selected credentials {}", selected_credentials);
        disclosed_proof::generate_proof(proof_handle, selected_credentials.into(), "{}".to_string()).unwrap();

        info!("generate_and_send_proof :: proof generated, sending proof");
        disclosed_proof::send_proof(proof_handle, connection_handle).unwrap();
        info!("generate_and_send_proof :: proof sent");

        assert_eq!(VcxStateType::VcxStateOfferSent as u32, disclosed_proof::get_state(proof_handle).unwrap());
        thread::sleep(Duration::from_millis(5000));
    }

    fn revoke_credential(issuer_handle: u32, rev_reg_id: Option<String>) {
        set_institution(None);
        // GET REV REG DELTA BEFORE REVOCATION
        let (_, delta, timestamp) = ::utils::libindy::anoncreds::get_rev_reg_delta_json(&rev_reg_id.clone().unwrap(), None, None).unwrap();
        info!("revoking credential");
        ::issuer_credential::revoke_credential(issuer_handle).unwrap();
        let (_, delta_after_revoke, _) = ::utils::libindy::anoncreds::get_rev_reg_delta_json(&rev_reg_id.unwrap(), Some(timestamp + 1), None).unwrap();
        assert_ne!(delta, delta_after_revoke);
    }

    fn revoke_credential_local(issuer_handle: u32, rev_reg_id: Option<String>) {
        set_institution(None);
        let (_, delta, timestamp) = ::utils::libindy::anoncreds::get_rev_reg_delta_json(&rev_reg_id.clone().unwrap(), None, None).unwrap();
        info!("revoking credential locally");
        ::issuer_credential::revoke_credential_local(issuer_handle).unwrap();
        let (_, delta_after_revoke, _) = ::utils::libindy::anoncreds::get_rev_reg_delta_json(&rev_reg_id.unwrap(), Some(timestamp + 1), None).unwrap();
        assert_ne!(delta, delta_after_revoke); // They will not equal as we have saved the delta in cache
    }

    fn publish_revocation(rev_reg_id: String) {
        set_institution(None);
        ::utils::libindy::anoncreds::publish_local_revocations(rev_reg_id.as_str()).unwrap();
    }

    fn _create_address_schema() -> (String, String, String, String, u32, Option<String>) {
        info!("test_real_proof_with_revocation >>> CREATE SCHEMA AND CRED DEF");
        let attrs_list = json!(["address1", "address2", "city", "state", "zip"]).to_string();
        ::utils::libindy::anoncreds::tests::create_and_store_credential_def(&attrs_list, true)
    }

    fn _exchange_credential(credential_data: String, cred_def_handle: u32, faber: u32, alice: u32, consumer_handle: Option<u32>) -> u32 {
        info!("Generated credential data: {}", credential_data);
        let credential_offer = create_and_send_cred_offer(settings::CONFIG_INSTITUTION_DID, cred_def_handle, alice, &credential_data);
        info!("AS CONSUMER SEND CREDENTIAL REQUEST");
        let credential = send_cred_req(faber, consumer_handle);
        info!("AS INSTITUTION SEND CREDENTIAL");
        send_credential(credential_offer, alice, credential, consumer_handle);
        credential_offer
    }

    fn _issue_address_credential(faber: u32, alice: u32, consumer_handle: Option<u32>) -> (String, String, Option<String>, u32, u32) {
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def_handle, rev_reg_id) = _create_address_schema();

        info!("test_real_proof_with_revocation :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data = json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"}).to_string();

        let credential_handle = _exchange_credential(credential_data, cred_def_handle, faber, alice, consumer_handle);
        (schema_id, cred_def_id, rev_reg_id, cred_def_handle, credential_handle)
    }

    fn _verifier_create_proof_and_send_request(institution_did: &str, schema_id: &str, cred_def_id: &str, alice: u32, institution_handle: Option<u32>, request_name: Option<&str>) -> u32 {
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, None);
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();
        send_proof_request(alice, &requested_attrs_string, "[]", "{}", institution_handle, request_name)
    }

    fn _prover_select_credentials_and_send_proof(faber: u32, consumer_handle: Option<u32>, request_name: Option<&str>) {
        set_consumer(consumer_handle);
        info!("Prover :: Going to create proof");
        let proof_handle_prover = create_proof(faber, consumer_handle, request_name);
        info!("Prover :: Retrieving matching credentials");
        let retrieved_credentials = disclosed_proof::retrieve_credentials(proof_handle_prover).unwrap();
        info!("Prover :: Based on proof, retrieved credentials: {}", &retrieved_credentials);
        let selected_credentials_value = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!("Prover :: Retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof(proof_handle_prover, faber, &selected_credentials_str, consumer_handle);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_basic_revocation() {
        let _setup = SetupLibraryAgencyV2::init();
        ::settings::set_config_value(::settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (faber, alice) = ::connection::tests::create_connected_connections(None, None);
        let (schema_id, cred_def_id, rev_reg_id, _cred_def_handle, credential_handle) = _issue_address_credential(faber, alice, None);

        let time_before_revocation = time::get_time().sec as u64;
        info!("test_basic_revocation :: verifier :: Going to revoke credential");
        revoke_credential(credential_handle, rev_reg_id);
        thread::sleep(Duration::from_millis(2000));
        let time_after_revocation = time::get_time().sec as u64;

        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, Some(time_after_revocation));
        let interval = json!({"from": time_before_revocation - 100, "to": time_after_revocation}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_basic_revocation :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let proof_handle_verifier = send_proof_request(alice, &requested_attrs_string, "[]", &interval, None, None);

        _prover_select_credentials_and_send_proof(faber, None, None);

        info!("test_basic_revocation :: verifier :: going to verify proof");
        set_institution(None);
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_local_revocation() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();

        set_institution(None);
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (consumer_to_institution, institution_to_consumer) = ::connection::tests::create_connected_connections(None, None);
        let (schema_id, cred_def_id, rev_reg_id, _cred_def_handle, credential_handle) = _issue_address_credential(consumer_to_institution, institution_to_consumer, None);

        revoke_credential_local(credential_handle, rev_reg_id.clone());
        let request_name1 = Some("request1");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer, None, request_name1);
        _prover_select_credentials_and_send_proof(consumer_to_institution, None, request_name1);

        set_institution(None);
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);

        publish_revocation(rev_reg_id.clone().unwrap());
        let request_name2 = Some("request2");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer, None, request_name2);
        _prover_select_credentials_and_send_proof(consumer_to_institution, None, request_name2);

        set_institution(None);
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_double_issuance_separate_issuer_and_consumers() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did

        let verifier = create_institution_config();
        let consumer1 = create_consumer_config();
        let consumer2 = create_consumer_config();
        let (consumer1_to_verifier, verifier_to_consumer1) = ::connection::tests::create_connected_connections(Some(consumer1), Some(verifier));
        let (consumer1_to_issuer, issuer_to_consumer1) = ::connection::tests::create_connected_connections(Some(consumer1), None); // Issuer
        let (consumer2_to_verifier, verifier_to_consumer2) = ::connection::tests::create_connected_connections(Some(consumer2), Some(verifier));
        let (consumer2_to_issuer, issuer_to_consumer2) = ::connection::tests::create_connected_connections(Some(consumer2), None); // Issuer


        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def_handle, rev_reg_id) = _create_address_schema();
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(credential_data1, cred_def_handle, consumer1_to_issuer, issuer_to_consumer1, Some(consumer1));
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(credential_data2, cred_def_handle, consumer2_to_issuer, issuer_to_consumer2, Some(consumer2));

        let request_name1 = Some("request1");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, verifier_to_consumer1, Some(verifier), request_name1);
        _prover_select_credentials_and_send_proof(consumer1_to_verifier, Some(consumer1), None);
        set_institution(Some(verifier));
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, verifier_to_consumer2, Some(verifier), request_name2);
        _prover_select_credentials_and_send_proof(consumer2_to_verifier, Some(consumer2), None);
        set_institution(Some(verifier));
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_double_issuance_separate_issuer() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did

        let verifier = create_institution_config();
        let consumer = create_consumer_config();
        let (consumer_to_verifier, verifier_to_consumer) = ::connection::tests::create_connected_connections(Some(consumer), Some(verifier));
        let (consumer_to_issuer, issuer_to_consumer) = ::connection::tests::create_connected_connections(Some(consumer), None); // Issuer

        let (schema_id, cred_def_id, rev_reg_id, _cred_def_handle, credential_handle) = _issue_address_credential(consumer_to_issuer, issuer_to_consumer, Some(consumer));
        let request_name1 = Some("request1");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, verifier_to_consumer, Some(verifier), request_name1);
        _prover_select_credentials_and_send_proof(consumer_to_verifier, Some(consumer), request_name1);
        set_institution(Some(verifier));
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, verifier_to_consumer, Some(verifier), request_name2);
        _prover_select_credentials_and_send_proof(consumer_to_verifier, Some(consumer), request_name2);
        set_institution(Some(verifier));
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_double_issuance_issuer_is_verifier() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let (consumer_to_institution, institution_to_consumer) = ::connection::tests::create_connected_connections(None, None);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def_handle, rev_reg_id) = _create_address_schema();
        let (address1, address, city, state, zip) = attr_names();
        let credential_data = json!({address1.clone(): "5th Avenue", address.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let credential_handle = _exchange_credential(credential_data, cred_def_handle, consumer_to_institution, institution_to_consumer, None);

        let request_name1 = Some("request1");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer, None, request_name1);
        _prover_select_credentials_and_send_proof(consumer_to_institution, None, request_name1);
        set_institution(None);
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let proof_handle_verifier = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer, None, request_name2);
        _prover_select_credentials_and_send_proof(consumer_to_institution, None, request_name2);
        set_institution(None);
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_batch_revocation() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let consumer1 = create_consumer_config();
        let consumer2 = create_consumer_config();
        let consumer3 = create_consumer_config();
        let (consumer_to_institution1, institution_to_consumer1) = ::connection::tests::create_connected_connections(Some(consumer1), None);
        let (consumer_to_institution2, institution_to_consumer2) = ::connection::tests::create_connected_connections(Some(consumer2), None);
        let (consumer_to_institution3, institution_to_consumer3) = ::connection::tests::create_connected_connections(Some(consumer3), None);
        assert_ne!(institution_to_consumer1, institution_to_consumer2); assert_ne!(institution_to_consumer1, institution_to_consumer3); assert_ne!(institution_to_consumer2, institution_to_consumer3);
        assert_ne!(consumer_to_institution1, consumer_to_institution2); assert_ne!(consumer_to_institution1, consumer_to_institution3); assert_ne!(consumer_to_institution2, consumer_to_institution3);

        // Issue and send three credentials of the same schema
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def_handle, rev_reg_id) = _create_address_schema();
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(credential_data1, cred_def_handle, consumer_to_institution1, institution_to_consumer1, Some(consumer1));
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(credential_data2, cred_def_handle, consumer_to_institution2, institution_to_consumer2, Some(consumer2));
        let credential_data3 = json!({address1.clone(): "5th Avenue", address2.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let credential_handle3 = _exchange_credential(credential_data3, cred_def_handle, consumer_to_institution3, institution_to_consumer3, Some(consumer3));

        revoke_credential_local(credential_handle1, rev_reg_id.clone());
        revoke_credential_local(credential_handle2, rev_reg_id.clone());

        // Revoke two locally and verify their are all still valid
        let request_name1 = Some("request1"); let request_name2 = Some("request2");
        let proof_handle_verifier1 = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer1, None, request_name1);
        _prover_select_credentials_and_send_proof(consumer_to_institution1, Some(consumer1), request_name1);
        let proof_handle_verifier2 = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer2, None, request_name1);
        _prover_select_credentials_and_send_proof(consumer_to_institution2, Some(consumer2), request_name1);
        let proof_handle_verifier3 = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer3, None, request_name1);
        _prover_select_credentials_and_send_proof(consumer_to_institution3, Some(consumer3), request_name1);

        set_institution(None);
        proof::update_state(proof_handle_verifier1, None, None).unwrap();
        proof::update_state(proof_handle_verifier2, None, None).unwrap();
        proof::update_state(proof_handle_verifier3, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier1).unwrap(), ProofStateType::ProofValidated as u32);
        assert_eq!(proof::get_proof_state(proof_handle_verifier2).unwrap(), ProofStateType::ProofValidated as u32);
        assert_eq!(proof::get_proof_state(proof_handle_verifier3).unwrap(), ProofStateType::ProofValidated as u32);

        // Publish revocations and verify the two are invalid, third still valid
        publish_revocation(rev_reg_id.clone().unwrap());
        thread::sleep(Duration::from_millis(2000));
        let request_name2 = Some("request2");
        let proof_handle_verifier1 = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer1, None, request_name2);
        _prover_select_credentials_and_send_proof(consumer_to_institution1, Some(consumer1), request_name2);
        let proof_handle_verifier2 = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer2, None, request_name2);
        _prover_select_credentials_and_send_proof(consumer_to_institution2, Some(consumer2), request_name2);
        let proof_handle_verifier3 = _verifier_create_proof_and_send_request(&institution_did, &schema_id, &cred_def_id, institution_to_consumer3, None, request_name2);
        _prover_select_credentials_and_send_proof(consumer_to_institution3, Some(consumer3), request_name2);
        assert_ne!(proof_handle_verifier1, proof_handle_verifier2); assert_ne!(proof_handle_verifier1, proof_handle_verifier3); assert_ne!(proof_handle_verifier2, proof_handle_verifier3);

        set_institution(None);
        proof::update_state(proof_handle_verifier1, None, None).unwrap();
        proof::update_state(proof_handle_verifier2, None, None).unwrap();
        proof::update_state(proof_handle_verifier3, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier1).unwrap(), ProofStateType::ProofInvalid as u32);
        assert_eq!(proof::get_proof_state(proof_handle_verifier2).unwrap(), ProofStateType::ProofInvalid as u32);
        assert_eq!(proof::get_proof_state(proof_handle_verifier3).unwrap(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_revoked_credential_might_still_work() {
        let _setup = SetupLibraryAgencyV2::init();
        ::settings::set_config_value(::settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (faber, alice) = ::connection::tests::create_connected_connections(None, None);
        let (schema_id, cred_def_id, rev_reg_id, _cred_def_handle, credential_handle) = _issue_address_credential(faber, alice, None);

        thread::sleep(Duration::from_millis(1000));
        let time_before_revocation = time::get_time().sec as u64;
        thread::sleep(Duration::from_millis(2000));
        info!("test_revoked_credential_might_still_work :: verifier :: Going to revoke credential");
        revoke_credential(credential_handle, rev_reg_id);
        thread::sleep(Duration::from_millis(2000));

        let from = time_before_revocation - 100;
        let to = time_before_revocation;
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, Some(from), Some(to));
        let interval = json!({"from": from, "to": to}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_revoked_credential_might_still_work :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let proof_handle_verifier = send_proof_request(alice, &requested_attrs_string, "[]", &interval, None, None);

        info!("test_revoked_credential_might_still_work :: Going to create proof");
        let proof_handle_prover = create_proof(faber, None, None);
        info!("test_revoked_credential_might_still_work :: retrieving matching credentials");

        let retrieved_credentials = disclosed_proof::retrieve_credentials(proof_handle_prover).unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: based on proof, retrieved credentials: {}", &retrieved_credentials);

        let selected_credentials_value = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof(proof_handle_prover, faber, &selected_credentials_str, None);

        info!("test_revoked_credential_might_still_work :: verifier :: going to verify proof");
        set_institution(None);
        proof::update_state(proof_handle_verifier, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_handle_verifier).unwrap(), ProofStateType::ProofValidated as u32);
    }

    fn retrieved_to_selected_credentials_simple(retrieved_credentials: &str, with_tails: bool) -> Value {
        info!("test_real_proof >>> retrieved matching credentials {}", retrieved_credentials);
        let data: Value = serde_json::from_str(retrieved_credentials).unwrap();
        let mut credentials_mapped: Value = json!({"attrs":{}, "predicates":{}});

        for (key, val) in data["attrs"].as_object().unwrap().iter() {
            let cred_array = val.as_array().unwrap();
            if cred_array.len() > 0 {
                let first_cred = &cred_array[0];
                credentials_mapped["attrs"][key]["credential"] = first_cred.clone();
                if with_tails {
                    credentials_mapped["attrs"][key]["tails_file"] = Value::from(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap());
                }
            }
        }
        return credentials_mapped;
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_real_proof() {
        let _setup = SetupLibraryAgencyV2::init();
        ::settings::set_config_value(::settings::CONFIG_PROTOCOL_TYPE, "4.0");

        info!("test_real_proof >>>");
        let number_of_attributes = 10;

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (faber, alice) = ::connection::tests::create_connected_connections(None, None);

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let mut attrs_list: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs_list.as_array_mut().unwrap().push(json!(format!("key{}",i)));
        }
        let attrs_list = attrs_list.to_string();
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def_handle, _) = ::utils::libindy::anoncreds::tests::create_and_store_credential_def(&attrs_list, false);
        let mut credential_data = json!({});
        for i in 1..number_of_attributes {
            credential_data[format!("key{}", i)] = Value::String(format!("value{}", i));
        }
        info!("test_real_proof :: sending credential offer");
        let credential_data = credential_data.to_string();
        info!("test_real_proof :: generated credential data: {}", credential_data);
        let credential_offer = create_and_send_cred_offer(&institution_did, cred_def_handle, alice, &credential_data);

        info!("test_real_proof :: AS CONSUMER SEND CREDENTIAL REQUEST");
        let credential = send_cred_req(faber, None);

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL");
        send_credential(credential_offer, alice, credential, None);

        info!("test_real_proof :: AS INSTITUTION SEND PROOF REQUEST");
        ::utils::devsetup::set_institution(None);

        let restrictions = json!({ "issuer_did": institution_did, "schema_id": schema_id, "cred_def_id": cred_def_id, });
        let mut attrs: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs.as_array_mut().unwrap().push(json!({ "name":format!("key{}", i), "restrictions": [restrictions]}));
        }
        let requested_attrs = attrs.to_string();
        info!("test_real_proof :: Going to seng proof request with attributes {}", requested_attrs);
        let proof_req_handle = send_proof_request(alice, &requested_attrs, "[]", "{}", None, None);

        info!("test_real_proof :: Going to create proof");
        let proof_handle = create_proof(faber, None, None);
        info!("test_real_proof :: retrieving matching credentials");

        let retrieved_credentials = disclosed_proof::retrieve_credentials(proof_handle).unwrap();
        let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, false);

        info!("test_real_proof :: generating and sending proof");
        generate_and_send_proof(proof_handle, faber, &serde_json::to_string(&selected_credentials).unwrap(), None);

        info!("test_real_proof :: AS INSTITUTION VALIDATE PROOF");
        set_institution(None);
        proof::update_state(proof_req_handle, None, None).unwrap();
        assert_eq!(proof::get_proof_state(proof_req_handle).unwrap(), ProofStateType::ProofValidated as u32);
    }
}
