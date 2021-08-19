#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "vcx"]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]
#[macro_use]
extern crate aries_vcx;
extern crate base64;
extern crate chrono;
extern crate failure;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate rand;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate time;
extern crate uuid;
extern crate tokio;

#[macro_use]
pub mod api_lib;
pub mod error;

#[allow(unused_imports)]
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use rand::Rng;
    use serde_json::Value;

    use crate::api_lib::api_handle::devsetup_agent::test::{Alice, Faber, TestAgent};
    use crate::api_lib::api_handle::test::create_connected_connections;
    use crate::api_lib::ProofStateType;
    use aries_vcx::settings;
    use aries_vcx::utils::{
        constants::{TEST_TAILS_FILE, TEST_TAILS_URL},
        get_temp_dir_path,
    };
    use aries_vcx::{libindy, utils};
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::handlers::issuance::holder::holder::{HolderState, Holder};
    use aries_vcx::handlers::issuance::issuer::issuer::{Issuer, IssuerConfig, IssuerState};
    use aries_vcx::handlers::issuance::credential_def::CredentialDef;
    use aries_vcx::handlers::issuance::holder::get_credential_offer_messages;
    use aries_vcx::handlers::proof_presentation::prover::prover::{ProverState, Prover};
    use aries_vcx::handlers::proof_presentation::verifier::verifier::{VerifierState, Verifier};
    use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
    use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
    use aries_vcx::handlers::proof_presentation::prover::get_proof_request_messages;
    use aries_vcx::handlers::connection::connection::Connection;
    use aries_vcx::utils::filters;

    use super::*;

    pub fn create_and_store_credential_def(attr_list: &str, support_rev: bool) -> (String, String, String, String, CredentialDef, Option<String>) {
        /* create schema */
        let (schema_id, schema_json) = libindy::utils::anoncreds::tests::create_and_write_test_schema(attr_list);

        let name: String = aries_vcx::utils::random::generate_random_name();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        /* create cred-def */
        let mut revocation_details = json!({"support_revocation":support_rev});
        if support_rev {
            revocation_details["tails_file"] = json!(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string());
            revocation_details["tails_url"] = json!(TEST_TAILS_URL);
            revocation_details["max_creds"] = json!(10);
        }
        let cred_def = CredentialDef::create("1".to_string(),
                                           name,
                                           institution_did.clone(),
                                           schema_id.clone(),
                                           "tag1".to_string(),
                                           revocation_details.to_string()).unwrap();

        thread::sleep(Duration::from_millis(1000));
        let cred_def_id = cred_def.get_cred_def_id();
        thread::sleep(Duration::from_millis(1000));
        let (_, cred_def_json) =  libindy::utils::anoncreds::get_cred_def_json(&cred_def_id).unwrap();
        let rev_reg_id = cred_def.get_rev_reg_id();
        (schema_id, schema_json, cred_def_id.to_string(), cred_def_json, cred_def, rev_reg_id)
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

    fn create_and_send_cred_offer(faber: &mut Faber, did: &str, cred_def: &CredentialDef, connection: &Connection, credential_data: &str, comment: Option<&str>) -> Issuer {
        faber.activate().unwrap();
        info!("create_and_send_cred_offer >> creating issuer credential");
        let issuer_config = IssuerConfig {
            cred_def_id: cred_def.get_cred_def_id(),
            rev_reg_id: cred_def.get_rev_reg_id(),
            tails_file: cred_def.get_tails_file(),
        };
        let mut issuer = Issuer::create(&issuer_config, &credential_data, "1").unwrap();
        info!("create_and_send_cred_offer :: sending credential offer");
        issuer.send_credential_offer(connection.send_message_closure().unwrap(), comment.map(|s| String::from(s))).unwrap();
        info!("create_and_send_cred_offer :: credential offer was sent");
        thread::sleep(Duration::from_millis(2000));
        issuer
    }

    fn send_cred_req(alice: &mut Alice, connection: &Connection, comment: Option<&str>) -> Holder {
        info!("send_cred_req >>> switching to consumer");
        alice.activate().unwrap();
        info!("send_cred_req :: getting offers");
        let credential_offers = get_credential_offer_messages(connection).unwrap();
        let credential_offers = match comment {
            Some(comment) => {
                let filtered = filters::filter_credential_offers_by_comment(&credential_offers, comment).unwrap();
                info!("send_cred_req :: credential offer  messages filtered by comment {}: {}", comment, filtered);
                filtered
            }
            _ => credential_offers.to_string()
        };
        let offers: Value = serde_json::from_str(&credential_offers).unwrap();
        let offers = offers.as_array().unwrap();
        assert_eq!(offers.len(), 1);
        let offer = serde_json::to_string(&offers[0]).unwrap();
        info!("send_cred_req :: creating credential from offer");
        let cred_offer: CredentialOffer = serde_json::from_str(&offer).unwrap();
        let mut holder = Holder::create(cred_offer, "TEST_CREDENTIAL").unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        info!("send_cred_req :: sending credential request");
        let my_pw_did = connection.pairwise_info().pw_did.to_string();
        holder.send_request(my_pw_did, connection.send_message_closure().unwrap()).unwrap();
        thread::sleep(Duration::from_millis(2000));
        holder
    }

    fn send_credential(consumer: &mut Alice, institution: &mut Faber, issuer_credential: &mut Issuer, issuer_to_consumer: &Connection, consumer_to_issuer: &Connection, holder_credential: &mut Holder, revokable: bool) {
        institution.activate().unwrap();
        info!("send_credential >>> getting offers");
        assert_eq!(issuer_credential.is_revokable().unwrap(), revokable);
        issuer_credential.update_state(issuer_to_consumer).unwrap();
        assert_eq!(IssuerState::RequestReceived, issuer_credential.get_state());
        assert_eq!(issuer_credential.is_revokable().unwrap(), revokable);

        info!("send_credential >>> sending credential");
        issuer_credential.send_credential(issuer_to_consumer.send_message_closure().unwrap()).unwrap();
        thread::sleep(Duration::from_millis(2000));
        
        consumer.activate().unwrap();
        info!("send_credential >>> storing credential");
        assert_eq!(holder_credential.is_revokable().unwrap(), revokable);
        holder_credential.update_state(consumer_to_issuer).unwrap();
        assert_eq!(HolderState::Finished, holder_credential.get_state());
        assert_eq!(holder_credential.is_revokable().unwrap(), revokable);

        if revokable {
            thread::sleep(Duration::from_millis(2000));
            assert_eq!(holder_credential.get_tails_location().unwrap(), TEST_TAILS_URL.to_string());
        }
    }

    fn send_proof_request(faber: &mut Faber, connection: &Connection, requested_attrs: &str, requested_preds: &str, revocation_interval: &str, request_name: Option<&str>) -> Verifier {
        faber.activate().unwrap();
        let mut verifier = Verifier::create("1".to_string(),
                                        requested_attrs.to_string(),
                                        requested_preds.to_string(),
                                        revocation_interval.to_string(),
                                        String::from(request_name.unwrap_or("name"))).unwrap();
        verifier.send_presentation_request(connection.send_message_closure().unwrap(), None).unwrap();
        thread::sleep(Duration::from_millis(2000));
        verifier
    }

    fn create_proof(alice: &mut Alice, connection: &Connection, request_name: Option<&str>) -> Prover {
        alice.activate().unwrap();
        info!("create_proof >>> getting proof request messages");
        let requests = {
            let _requests = get_proof_request_messages(connection).unwrap();
            info!("create_proof :: get proof request messages returned {}", _requests);
            match request_name {
                Some(request_name) => {
                    let filtered = filters::filter_proof_requests_by_name(&_requests, request_name).unwrap();
                    info!("create_proof :: proof request messages filtered by name {}: {}", request_name, filtered);
                    filtered
                }
                _ => _requests.to_string()
            }
        };
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = requests.as_array().unwrap();
        assert_eq!(requests.len(), 1);
        let request = serde_json::to_string(&requests[0]).unwrap();
        let presentation_request: PresentationRequest = serde_json::from_str(&request).unwrap();
        Prover::create(utils::constants::DEFAULT_PROOF_NAME, presentation_request).unwrap()
    }

    fn generate_and_send_proof(alice: &mut Alice, prover: &mut Prover, connection: &Connection, selected_credentials: &str) {
        alice.activate().unwrap();
        info!("generate_and_send_proof >>> generating proof using selected credentials {}", selected_credentials);
        prover.generate_presentation(selected_credentials.into(), "{}".to_string()).unwrap();

        info!("generate_and_send_proof :: proof generated, sending proof");
        prover.send_presentation(&connection.send_message_closure().unwrap()).unwrap();
        info!("generate_and_send_proof :: proof sent");

        assert_eq!(ProverState::PresentationSent, prover.get_state());
        thread::sleep(Duration::from_millis(5000));
    }

    fn revoke_credential(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: Option<String>) {
        faber.activate().unwrap();
        // GET REV REG DELTA BEFORE REVOCATION
        let (_, delta, timestamp) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.clone().unwrap(), None, None).unwrap();
        info!("revoking credential");
        issuer_credential.revoke_credential(true).unwrap();
        let (_, delta_after_revoke, _) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.unwrap(), Some(timestamp + 1), None).unwrap();
        assert_ne!(delta, delta_after_revoke);
    }

    fn revoke_credential_local(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: Option<String>) {
        faber.activate().unwrap();
        let (_, delta, timestamp) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.clone().unwrap(), None, None).unwrap();
        info!("revoking credential locally");
        issuer_credential.revoke_credential(false).unwrap();
        let (_, delta_after_revoke, _) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.unwrap(), Some(timestamp + 1), None).unwrap();
        assert_ne!(delta, delta_after_revoke); // They will not equal as we have saved the delta in cache
    }

    fn rotate_rev_reg(faber: &mut Faber, cred_def: &mut CredentialDef) {
        faber.activate().unwrap();
        let revocation_details = json!({
            "tails_file": json!(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            "tails_url": json!(TEST_TAILS_URL),
            "max_creds": json!(10)
        }).to_string();
        cred_def.rotate_rev_reg(&revocation_details).unwrap();
    }

    fn publish_revocation(institution: &mut Faber, rev_reg_id: String) {
        institution.activate().unwrap();
        libindy::utils::anoncreds::publish_local_revocations(rev_reg_id.as_str()).unwrap();
    }

    fn _create_address_schema() -> (String, String, String, String, CredentialDef, Option<String>) {
        info!("test_real_proof_with_revocation >>> CREATE SCHEMA AND CRED DEF");
        let attrs_list = json!(["address1", "address2", "city", "state", "zip"]).to_string();
        create_and_store_credential_def(&attrs_list, true)
    }

    fn _exchange_credential(consumer: &mut Alice, institution: &mut Faber, credential_data: String, cred_def: &CredentialDef, consumer_to_issuer: &Connection, issuer_to_consumer: &Connection, comment: Option<&str>) -> Issuer {
        info!("Generated credential data: {}", credential_data);
        let mut issuer_credential = create_and_send_cred_offer(institution, settings::CONFIG_INSTITUTION_DID, cred_def, issuer_to_consumer, &credential_data, comment);
        info!("AS CONSUMER SEND CREDENTIAL REQUEST");
        let mut holder_credential = send_cred_req(consumer, consumer_to_issuer, comment);
        info!("AS INSTITUTION SEND CREDENTIAL");
        send_credential(consumer, institution, &mut issuer_credential, issuer_to_consumer, consumer_to_issuer, &mut holder_credential, true);
        issuer_credential
    }

    fn _issue_address_credential(consumer: &mut Alice, institution: &mut Faber, consumer_to_institution: &Connection, institution_to_consumer: &Connection) -> (String, String, Option<String>, CredentialDef, Issuer) {
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema();

        info!("test_real_proof_with_revocation :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data = json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"}).to_string();

        let credential_handle = _exchange_credential(consumer, institution, credential_data, &cred_def, consumer_to_institution, institution_to_consumer, None);
        (schema_id, cred_def_id, rev_reg_id, cred_def, credential_handle)
    }

    fn _verifier_create_proof_and_send_request(institution: &mut Faber, institution_to_consumer: &Connection, schema_id: &str, cred_def_id: &str, request_name: Option<&str>) -> Verifier {
        institution.activate().unwrap();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, None);
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();
        send_proof_request(institution, institution_to_consumer, &requested_attrs_string, "[]", "{}", request_name)
    }

    fn _prover_select_credentials_and_send_proof(consumer: &mut Alice, consumer_to_institution: &Connection, request_name: Option<&str>, requested_values: Option<&str>) {
        consumer.activate().unwrap();
        info!("Prover :: Going to create proof");
        let mut prover = create_proof(consumer, consumer_to_institution, request_name);
        info!("Prover :: Retrieving matching credentials");
        let retrieved_credentials = prover.retrieve_credentials().unwrap();
        info!("Prover :: Based on proof, retrieved credentials: {}", &retrieved_credentials);
        let selected_credentials_value = match requested_values {
            Some(requested_values) => {
                let credential_data = prover.presentation_request_data().unwrap();
                retrieved_to_selected_credentials_specific(&retrieved_credentials, requested_values, &credential_data, true)
            }
            _ => retrieved_to_selected_credentials_simple(&retrieved_credentials, true)
        };
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!("Prover :: Retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof(consumer, &mut prover, consumer_to_institution, &selected_credentials_str);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_basic_revocation() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut institution = Faber::setup();
        let mut consumer = Alice::setup();

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution);
        let (schema_id, cred_def_id, rev_reg_id, _cred_def, credential_handle) = _issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer);

        let time_before_revocation = time::get_time().sec as u64;
        info!("test_basic_revocation :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg_id);
        thread::sleep(Duration::from_millis(2000));
        let time_after_revocation = time::get_time().sec as u64;

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, Some(time_after_revocation));
        let interval = json!({"from": time_before_revocation - 100, "to": time_after_revocation}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_basic_revocation :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None);

        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None);

        info!("test_basic_revocation :: verifier :: going to verify proof");
        institution.activate().unwrap();
        verifier.update_state(&institution_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_local_revocation() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut institution = Faber::setup();
        let mut consumer = Alice::setup();

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution);
        let (schema_id, cred_def_id, rev_reg_id, _cred_def, issuer_credential) = _issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer);

        revoke_credential_local(&mut institution, &issuer_credential, rev_reg_id.clone());
        let request_name1 = Some("request1");
        let mut verifier = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None);

        institution.activate().unwrap();
        verifier.update_state(&institution_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        publish_revocation(&mut institution, rev_reg_id.clone().unwrap());
        let request_name2 = Some("request2");
        let mut verifier = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None);

        institution.activate().unwrap();
        verifier.update_state(&institution_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_double_issuance_separate_issuer_and_consumers() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer1 = Alice::setup();
        let mut consumer2 = Alice::setup();
        let (consumer1_to_verifier, verifier_to_consumer1) = create_connected_connections(&mut consumer1, &mut verifier);
        let (consumer1_to_issuer, issuer_to_consumer1) = create_connected_connections(&mut consumer1, &mut issuer);
        let (consumer2_to_verifier, verifier_to_consumer2) = create_connected_connections(&mut consumer2, &mut verifier);
        let (consumer2_to_issuer, issuer_to_consumer2) = create_connected_connections(&mut consumer2, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _rev_reg_id) = _create_address_schema();
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer1, &mut issuer, credential_data1, &cred_def, &consumer1_to_issuer, &issuer_to_consumer1, None);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer2, &mut issuer, credential_data2, &cred_def,  &consumer2_to_issuer, &issuer_to_consumer2, None);

        let request_name1 = Some("request1");
        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer1, &schema_id, &cred_def_id, request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer1, &consumer1_to_verifier, None, None);
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer1).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer2, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer2, &consumer2_to_verifier, None, None);
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer2).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }


    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_double_issuance_separate_issuer() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();

        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _credential_handle) = _issue_address_credential(&mut consumer, &mut issuer, &consumer_to_issuer, &issuer_to_consumer);
        issuer.activate().unwrap();
        let request_name1 = Some("request1");
        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name1, None);
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name2, None);
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_double_issuance_issuer_is_verifier() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut institution = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _rev_reg_id) = _create_address_schema();
        let (address1, address, city, state, zip) = attr_names();
        let credential_data = json!({address1.clone(): "5th Avenue", address.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle = _exchange_credential(&mut consumer, &mut institution, credential_data, &cred_def, &consumer_to_institution, &institution_to_consumer, None);

        let request_name1 = Some("request1");
        let mut verifier = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None);
        institution.activate().unwrap();
        verifier.update_state(&institution_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut verifier = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None);
        institution.activate().unwrap();
        verifier.update_state(&institution_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_batch_revocation() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let mut institution = Faber::setup();
        let mut consumer1 = Alice::setup();
        let mut consumer2 = Alice::setup();
        let mut consumer3 = Alice::setup();
        let (consumer_to_institution1, institution_to_consumer1) = create_connected_connections(&mut consumer1, &mut institution);
        let (consumer_to_institution2, institution_to_consumer2) = create_connected_connections(&mut consumer2, &mut institution);
        let (consumer_to_institution3, institution_to_consumer3) = create_connected_connections(&mut consumer3, &mut institution);
        // assert_ne!(institution_to_consumer1, institution_to_consumer2);
        // assert_ne!(institution_to_consumer1, institution_to_consumer3);
        // assert_ne!(institution_to_consumer2, institution_to_consumer3);
        // assert_ne!(consumer_to_institution1, consumer_to_institution2);
        // assert_ne!(consumer_to_institution1, consumer_to_institution3);
        // assert_ne!(consumer_to_institution2, consumer_to_institution3);

        // Issue and send three credentials of the same schema
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema();
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer1, &mut institution, credential_data1, &cred_def, &consumer_to_institution1, &institution_to_consumer1, None);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer2, &mut institution, credential_data2, &cred_def, &consumer_to_institution2, &institution_to_consumer2, None);
        let credential_data3 = json!({address1.clone(): "5th Avenue", address2.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let credential_handle3 = _exchange_credential(&mut consumer3, &mut institution, credential_data3, &cred_def, &consumer_to_institution3, &institution_to_consumer3, None);

        revoke_credential_local(&mut institution, &credential_handle1, rev_reg_id.clone());
        revoke_credential_local(&mut institution, &credential_handle2, rev_reg_id.clone());

        // Revoke two locally and verify their are all still valid
        let request_name1 = Some("request1");
        let mut verifier1 = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name1, None);
        let mut verifier2 = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name1, None);
        let mut verifier3 = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name1);
        _prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name1, None);

        institution.activate().unwrap();
        verifier1.update_state(&institution_to_consumer1).unwrap();
        verifier2.update_state(&institution_to_consumer2).unwrap();
        verifier3.update_state(&institution_to_consumer3).unwrap();
        assert_eq!(verifier1.presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(verifier2.presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(verifier3.presentation_status(), ProofStateType::ProofValidated as u32);

        // Publish revocations and verify the two are invalid, third still valid
        publish_revocation(&mut institution, rev_reg_id.clone().unwrap());
        thread::sleep(Duration::from_millis(2000));
        let request_name2 = Some("request2");
        let mut verifier1 = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1,  request_name2, None);
        let mut verifier2 = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2,  request_name2, None);
        let mut verifier3 = _verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name2);
        _prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3,  request_name2, None);
        assert_ne!(verifier1, verifier2);
        assert_ne!(verifier1, verifier3);
        assert_ne!(verifier2, verifier3);

        institution.activate().unwrap();
        verifier1.update_state(&institution_to_consumer1).unwrap();
        verifier2.update_state(&institution_to_consumer2).unwrap();
        verifier3.update_state(&institution_to_consumer3).unwrap();
        assert_eq!(verifier1.presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier2.presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier3.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_revoked_credential_might_still_work() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut institution = Faber::setup();
        let mut consumer = Alice::setup();

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution);
        let (schema_id, cred_def_id, rev_reg_id, _cred_def, credential_handle) = _issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer);

        thread::sleep(Duration::from_millis(1000));
        let time_before_revocation = time::get_time().sec as u64;
        thread::sleep(Duration::from_millis(2000));
        info!("test_revoked_credential_might_still_work :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg_id);
        thread::sleep(Duration::from_millis(2000));

        let from = time_before_revocation - 100;
        let to = time_before_revocation;
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, Some(from), Some(to));
        let interval = json!({"from": from, "to": to}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_revoked_credential_might_still_work :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None);

        info!("test_revoked_credential_might_still_work :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_institution, None);
        info!("test_revoked_credential_might_still_work :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials().unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: based on proof, retrieved credentials: {}", &retrieved_credentials);

        let selected_credentials_value = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str);

        info!("test_revoked_credential_might_still_work :: verifier :: going to verify proof");
        institution.activate().unwrap();
        verifier.update_state(&institution_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofValidated as u32);
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

    fn retrieved_to_selected_credentials_specific(retrieved_credentials: &str, requested_values: &str, credential_data: &str, with_tails: bool) -> Value {
        info!("test_real_proof >>> retrieved matching credentials {}", retrieved_credentials);
        let retrieved_credentials: Value = serde_json::from_str(retrieved_credentials).unwrap();
        let credential_data: Value = serde_json::from_str(credential_data).unwrap();
        let requested_values: Value = serde_json::from_str(requested_values).unwrap();
        let requested_attributes: &Value = &credential_data["requested_attributes"];
        let mut credentials_mapped: Value = json!({"attrs":{}, "predicates":{}});

        for (key, val) in retrieved_credentials["attrs"].as_object().unwrap().iter() {
            let filtered: Vec<&Value> = val.as_array().unwrap()
                .into_iter()
                .filter_map(|cred| {
                    let attribute_name = requested_attributes[key]["name"].as_str().unwrap();
                    let requested_value = requested_values[attribute_name].as_str().unwrap();
                    if cred["cred_info"]["attrs"][attribute_name].as_str().unwrap() == requested_value {
                        Some(cred)
                    } else {
                        None
                    }
                })
                .collect();
            let first_cred: &serde_json::Value = &filtered[0];
            credentials_mapped["attrs"][key]["credential"] = first_cred.clone();
            if with_tails {
                credentials_mapped["attrs"][key]["tails_file"] = Value::from(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap());
            }
        }
        return credentials_mapped;
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_real_proof() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut institution = Faber::setup();
        let mut consumer = Alice::setup();

        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut institution);

        info!("test_real_proof >>>");
        let number_of_attributes = 10;

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let mut attrs_list: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs_list.as_array_mut().unwrap().push(json!(format!("key{}",i)));
        }
        let attrs_list = attrs_list.to_string();
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _) = create_and_store_credential_def(&attrs_list, false);
        let mut credential_data = json!({});
        for i in 1..number_of_attributes {
            credential_data[format!("key{}", i)] = Value::String(format!("value{}", i));
        }
        info!("test_real_proof :: sending credential offer");
        let credential_data = credential_data.to_string();
        info!("test_real_proof :: generated credential data: {}", credential_data);
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let mut issuer_credential = create_and_send_cred_offer(&mut institution, &institution_did, &cred_def, &issuer_to_consumer, &credential_data, None);

        info!("test_real_proof :: AS CONSUMER SEND CREDENTIAL REQUEST");
        let mut holder_credential = send_cred_req(&mut consumer, &consumer_to_issuer, None);

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL");
        send_credential(&mut consumer, &mut institution, &mut issuer_credential, &issuer_to_consumer, &consumer_to_issuer, &mut holder_credential, false);

        info!("test_real_proof :: AS INSTITUTION SEND PROOF REQUEST");
        institution.activate().unwrap();

        let restrictions = json!({ "issuer_did": institution_did, "schema_id": schema_id, "cred_def_id": cred_def_id, });
        let mut attrs: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs.as_array_mut().unwrap().push(json!({ "name":format!("key{}", i), "restrictions": [restrictions]}));
        }
        let requested_attrs = attrs.to_string();
        info!("test_real_proof :: Going to seng proof request with attributes {}", requested_attrs);
        let mut verifier = send_proof_request(&mut institution, &issuer_to_consumer, &requested_attrs, "[]", "{}", None);

        info!("test_real_proof :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_issuer, None);
        info!("test_real_proof :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials().unwrap();
        let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, false);

        info!("test_real_proof :: generating and sending proof");
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_issuer, &serde_json::to_string(&selected_credentials).unwrap());

        info!("test_real_proof :: AS INSTITUTION VALIDATE PROOF");
        institution.activate().unwrap();
        verifier.update_state(&issuer_to_consumer).unwrap();
        assert_eq!(verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_two_creds_one_rev_reg() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _rev_reg_id) = _create_address_schema();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer,credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }
    //
    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_two_creds_one_rev_reg_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer,  req1);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2);

        revoke_credential(&mut issuer, &credential_handle1, rev_reg_id);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofInvalid as u32);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_two_creds_one_rev_reg_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer,  req1);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2);

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_id);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_two_creds_two_rev_reg_id() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg_id) = _create_address_schema();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer,  req1);
        rotate_rev_reg(&mut issuer, &mut cred_def);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_two_creds_two_rev_reg_id_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg_id) = _create_address_schema();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer,  req1);
        rotate_rev_reg(&mut issuer, &mut cred_def);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2);

        revoke_credential(&mut issuer, &credential_handle1, rev_reg_id);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofInvalid as u32);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    fn test_two_creds_two_rev_reg_id_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init();
        let mut issuer = Faber::setup();
        let mut verifier = Faber::setup();
        let mut consumer = Alice::setup();
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier);
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer);

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg_id) = _create_address_schema();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(); // Issuer's did
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer,  req1);
        rotate_rev_reg(&mut issuer, &mut cred_def);
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2);

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_id);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req1);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = _verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id,  req2);
        _prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2));
        verifier.activate().unwrap();
        proof_verifier.update_state(&verifier_to_consumer).unwrap();
        assert_eq!(proof_verifier.presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[test]
    #[cfg(feature = "agency_pool_tests")]
    pub fn test_two_enterprise_connections() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let mut institution = Faber::setup();
        let mut consumer1 = Alice::setup();

        let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution);
        let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution);
    }
}
