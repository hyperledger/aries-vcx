extern crate async_trait;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;

pub mod utils;

#[allow(unused_imports)]
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::fmt;
    use std::ops::Deref;
    use std::thread;
    use std::time::Duration;

    use indyrs::wallet;
    use rand::Rng;
    use serde_json::Value;

    use agency_client::agency_client::AgencyClient;
    use agency_client::api::downloaded_message::DownloadedMessage;
    use agency_client::messages::update_message::UIDsByConn;
    use aries_vcx::{libindy, utils};
    use aries_vcx::agency_client::MessageStatusCode;
    use aries_vcx::error::VcxResult;
    use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::issuer::{Issuer, IssuerConfig};
    use aries_vcx::handlers::issuance::issuer::test_utils::get_credential_proposal_messages;
    use aries_vcx::handlers::out_of_band::{GoalCode, HandshakeProtocol, OutOfBandInvitation};
    use aries_vcx::handlers::out_of_band::receiver::OutOfBandReceiver;
    use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::libindy::wallet::open_wallet;
    use aries_vcx::libindy::credential_def;
    use aries_vcx::libindy::credential_def::{CredentialDef, CredentialDefConfigBuilder, RevocationDetailsBuilder};
    use aries_vcx::libindy::credential_def::revocation_registry::RevocationRegistry;
    use aries_vcx::libindy::proofs::proof_request_internal::{AttrInfo, NonRevokedInterval, PredicateInfo};
    use aries_vcx::libindy::utils::anoncreds::test_utils::{create_and_store_credential_def, create_and_store_nonrevocable_credential_def, create_and_write_test_schema};
    use aries_vcx::libindy::utils::signus;
    use aries_vcx::libindy::utils::signus::create_and_store_my_did;
    use aries_vcx::libindy::utils::wallet::*;
    use aries_vcx::messages::a2a::A2AMessage;
    use aries_vcx::messages::ack::test_utils::_ack;
    use aries_vcx::messages::connection::invite::Invitation;
    use aries_vcx::messages::connection::service::FullService;
    use aries_vcx::messages::connection::service::ServiceResolvable;
    use aries_vcx::messages::issuance::credential_offer::{CredentialOffer, OfferInfo};
    use aries_vcx::messages::issuance::credential_proposal::{CredentialProposal, CredentialProposalData};
    use aries_vcx::messages::mime_type::MimeType;
    use aries_vcx::messages::proof_presentation::presentation_proposal::{Attribute, PresentationProposal, PresentationProposalData};
    use aries_vcx::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
    use aries_vcx::protocols::connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::connection::inviter::state_machine::InviterState;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::global::settings;
    use aries_vcx::global::agency_client::get_main_agency_client;
    use aries_vcx::global::wallet::get_main_wallet_handle;
    use aries_vcx::utils::{
        constants::{TAILS_DIR, TEST_TAILS_URL},
        get_temp_dir_path,
    };
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::filters;
    use aries_vcx::utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED, CONNECTION_SM_INVITER_COMPLETED};
    use aries_vcx::utils::mockdata::mockdata_proof::REQUESTED_ATTRIBUTES;

    use crate::utils::devsetup_agent::test::{Alice, Faber, TestAgent, PayloadKinds};
    use crate::utils::scenarios::test_utils::{_create_address_schema, _exchange_credential, _exchange_credential_with_proposal, accept_cred_proposal, accept_cred_proposal_1, accept_offer, accept_proof_proposal, attr_names, connect_using_request_sent_to_public_agent, create_and_send_nonrevocable_cred_offer, create_connected_connections, create_connected_connections_via_public_invite, create_proof, create_proof_request, decline_offer, generate_and_send_proof_boo, issue_address_credential, prover_select_credentials, prover_select_credentials_and_fail_to_generate_proof, prover_select_credentials_and_send_proof, publish_revocation, receive_proof_proposal_rejection, reject_proof_proposal, requested_attrs, retrieved_to_selected_credentials_simple, revoke_credential, revoke_credential_local, rotate_rev_reg, send_cred_proposal, send_cred_proposal_1, send_cred_req, send_credential, send_proof_proposal, send_proof_proposal_1, send_proof_request, verifier_create_proof_and_send_request, verify_proof};
    use crate::utils::test_macros::ProofStateType;

    use super::*;

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_proof_should_be_validated() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _rev_reg, _credential_handle) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;
        institution.activate().await.unwrap();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let requested_attrs_string = serde_json::to_string(&json!([
           {
               "name": "address1",
               "restrictions": [{
                 "issuer_did": institution_did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }]
           }])).unwrap();


        info!("test_proof_should_be_validated :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", "{}", None).await;

        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

        info!("test_proof_should_be_validated :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_proof_with_predicates_should_be_validated() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;
        institution.activate().await.unwrap();
        let requested_preds_string = serde_json::to_string(&json!([
           {
               "name": "zip",
               "p_type": ">=",
               "p_value": 83000
           }])).unwrap();

        info!("test_basic_proof :: Going to seng proof request with attributes {}", &requested_preds_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, "[]", &requested_preds_string, "{}", None).await;

        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

        info!("test_basic_revocation :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
        info!("verifier received presentation!: {}", verifier.get_presentation_attachment().unwrap());
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_it_should_fail_to_select_credentials_for_predicate() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;
        institution.activate().await.unwrap();
        let requested_preds_string = serde_json::to_string(&json!([
           {
               "name": "zip",
               "p_type": ">=",
               "p_value": 85000
           }])).unwrap();

        info!("test_basic_proof :: Going to seng proof request with attributes {}", &requested_preds_string);
        send_proof_request(&mut institution, &institution_to_consumer, "[]", &requested_preds_string, "{}", None).await;

        prover_select_credentials_and_fail_to_generate_proof(&mut consumer, &consumer_to_institution, None, None).await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_basic_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _, _cred_def, rev_reg, credential_handle) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

        let time_before_revocation = time::get_time().sec as u64;
        info!("test_basic_revocation :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg.rev_reg_id).await;
        thread::sleep(Duration::from_millis(2000));
        let time_after_revocation = time::get_time().sec as u64;

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, Some(time_after_revocation));
        let interval = json!({"from": time_before_revocation - 100, "to": time_after_revocation}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_basic_revocation :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None).await;

        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

        info!("test_basic_revocation :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_local_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _, _cred_def, rev_reg, issuer_credential) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

        revoke_credential_local(&mut institution, &issuer_credential, rev_reg.rev_reg_id.clone()).await;
        let request_name1 = Some("request1");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None).await;

        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        publish_revocation(&mut institution, rev_reg.rev_reg_id.clone()).await;
        let request_name2 = Some("request2");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;

        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_double_issuance_separate_issuer_and_consumers() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let (consumer1_to_verifier, verifier_to_consumer1) = create_connected_connections(&mut consumer1, &mut verifier).await;
        let (consumer1_to_issuer, issuer_to_consumer1) = create_connected_connections(&mut consumer1, &mut issuer).await;
        let (consumer2_to_verifier, verifier_to_consumer2) = create_connected_connections(&mut consumer2, &mut verifier).await;
        let (consumer2_to_issuer, issuer_to_consumer2) = create_connected_connections(&mut consumer2, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer1, &mut issuer, credential_data1, &cred_def, &rev_reg, &consumer1_to_issuer, &issuer_to_consumer1, None).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer2, &mut issuer, credential_data2, &cred_def, &rev_reg, &consumer2_to_issuer, &issuer_to_consumer2, None).await;

        let request_name1 = Some("request1");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer1, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer1_to_verifier, None, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer1).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer2, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer2_to_verifier, None, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer2).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_double_issuance_separate_issuer() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _rev_reg, _credential_handle) = issue_address_credential(&mut consumer, &mut issuer, &consumer_to_issuer, &issuer_to_consumer).await;
        issuer.activate().await.unwrap();
        let request_name1 = Some("request1");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name1, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name2, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_double_issuance_issuer_is_verifier() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let (address1, address, city, state, zip) = attr_names();
        let credential_data = json!({address1.clone(): "5th Avenue", address.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle = _exchange_credential(&mut consumer, &mut institution, credential_data, &cred_def, &rev_reg, &consumer_to_institution, &institution_to_consumer, None).await;

        let request_name1 = Some("request1");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None).await;
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_batch_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let mut consumer3 = Alice::setup().await;
        let (consumer_to_institution1, institution_to_consumer1) = create_connected_connections(&mut consumer1, &mut institution).await;
        let (consumer_to_institution2, institution_to_consumer2) = create_connected_connections(&mut consumer2, &mut institution).await;
        let (consumer_to_institution3, institution_to_consumer3) = create_connected_connections(&mut consumer3, &mut institution).await;

        // Issue and send three credentials of the same schema
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer1, &mut institution, credential_data1, &cred_def, &rev_reg, &consumer_to_institution1, &institution_to_consumer1, None).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer2, &mut institution, credential_data2, &cred_def, &rev_reg, &consumer_to_institution2, &institution_to_consumer2, None).await;
        let credential_data3 = json!({address1.clone(): "5th Avenue", address2.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle3 = _exchange_credential(&mut consumer3, &mut institution, credential_data3, &cred_def, &rev_reg, &consumer_to_institution3, &institution_to_consumer3, None).await;

        revoke_credential_local(&mut institution, &credential_handle1, rev_reg.rev_reg_id.clone()).await;
        revoke_credential_local(&mut institution, &credential_handle2, rev_reg.rev_reg_id.clone()).await;

        // Revoke two locally and verify their are all still valid
        let request_name1 = Some("request1");
        let mut verifier1 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name1, None).await;
        let mut verifier2 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name1, None).await;
        let mut verifier3 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name1, None).await;

        institution.activate().await.unwrap();
        verifier1.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer1).await.unwrap();
        verifier2.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer2).await.unwrap();
        verifier3.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer3).await.unwrap();
        assert_eq!(verifier1.get_presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(verifier2.get_presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(verifier3.get_presentation_status(), ProofStateType::ProofValidated as u32);

        // Publish revocations and verify the two are invalid, third still valid
        publish_revocation(&mut institution, rev_reg_id.clone().unwrap()).await;
        thread::sleep(Duration::from_millis(2000));
        let request_name2 = Some("request2");
        let mut verifier1 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name2, None).await;
        let mut verifier2 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name2, None).await;
        let mut verifier3 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name2, None).await;
        assert_ne!(verifier1, verifier2);
        assert_ne!(verifier1, verifier3);
        assert_ne!(verifier2, verifier3);

        institution.activate().await.unwrap();
        verifier1.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer1).await.unwrap();
        verifier2.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer2).await.unwrap();
        verifier3.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer3).await.unwrap();
        assert_eq!(verifier1.get_presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier2.get_presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier3.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_revoked_credential_might_still_work() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _, _cred_def, rev_reg, credential_handle) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

        thread::sleep(Duration::from_millis(1000));
        let time_before_revocation = time::get_time().sec as u64;
        thread::sleep(Duration::from_millis(2000));
        info!("test_revoked_credential_might_still_work :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg.rev_reg_id.clone()).await;
        thread::sleep(Duration::from_millis(2000));

        let from = time_before_revocation - 100;
        let to = time_before_revocation;
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, Some(from), Some(to));
        let interval = json!({"from": from, "to": to}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_revoked_credential_might_still_work :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None).await;

        info!("test_revoked_credential_might_still_work :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_institution, None).await;
        info!("test_revoked_credential_might_still_work :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials(get_main_wallet_handle()).await.unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: based on proof, retrieved credentials: {}", &retrieved_credentials);

        let selected_credentials_value = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof_boo(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
        assert_eq!(ProverState::PresentationSent, prover.get_state());

        info!("test_revoked_credential_might_still_work :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_real_proof() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;

        info!("test_real_proof >>>");
        let number_of_attributes = 10;

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let mut attrs_list: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs_list.as_array_mut().unwrap().push(json!(format!("key{}",i)));
        }
        let attrs_list = attrs_list.to_string();
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def) = create_and_store_nonrevocable_credential_def(institution.wallet_handle, &attrs_list).await;
        let mut credential_data = json!({});
        for i in 1..number_of_attributes {
            credential_data[format!("key{}", i)] = Value::String(format!("value{}", i));
        }
        info!("test_real_proof :: sending credential offer");
        let credential_data = credential_data.to_string();
        info!("test_real_proof :: generated credential data: {}", credential_data);
        let mut issuer_credential = create_and_send_nonrevocable_cred_offer(&mut institution, &cred_def, &issuer_to_consumer, &credential_data, None).await;
        let issuance_thread_id = issuer_credential.get_thread_id().unwrap();

        info!("test_real_proof :: AS CONSUMER SEND CREDENTIAL REQUEST");
        let mut holder_credential = send_cred_req(&mut consumer, &consumer_to_issuer, None).await;

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL");
        send_credential(&mut consumer, &mut institution, &mut issuer_credential, &issuer_to_consumer, &consumer_to_issuer, &mut holder_credential, false).await;
        assert_eq!(issuance_thread_id, holder_credential.get_thread_id().unwrap());
        assert_eq!(issuance_thread_id, issuer_credential.get_thread_id().unwrap());

        info!("test_real_proof :: AS INSTITUTION SEND PROOF REQUEST");
        institution.activate().await.unwrap();

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let restrictions = json!({ "issuer_did": institution_did, "schema_id": schema_id, "cred_def_id": cred_def_id, });
        let mut attrs: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs.as_array_mut().unwrap().push(json!({ "name":format!("key{}", i), "restrictions": [restrictions]}));
        }
        let requested_attrs = attrs.to_string();
        info!("test_real_proof :: Going to seng proof request with attributes {}", requested_attrs);
        let mut verifier = send_proof_request(&mut institution, &issuer_to_consumer, &requested_attrs, "[]", "{}", None).await;
        let presentation_thread_id = verifier.get_thread_id().unwrap();

        info!("test_real_proof :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_issuer, None).await;
        info!("test_real_proof :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials(get_main_wallet_handle()).await.unwrap();
        let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, false);

        info!("test_real_proof :: generating and sending proof");
        generate_and_send_proof_boo(&mut consumer, &mut prover, &consumer_to_issuer, &serde_json::to_string(&selected_credentials).unwrap()).await;
        assert_eq!(ProverState::PresentationSent, prover.get_state());
        assert_eq!(presentation_thread_id, prover.get_thread_id().unwrap());
        assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());

        info!("test_real_proof :: AS INSTITUTION VALIDATE PROOF");
        institution.activate().await.unwrap();
        verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &issuer_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, _rev_reg_id) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle1, rev_reg_id.unwrap()).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_id.unwrap()).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg, _) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &rev_reg_2, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg, _) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &rev_reg_2, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle1, rev_reg.rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg, _) = _create_address_schema(issuer.wallet_handle).await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &rev_reg, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let rev_reg_2 = rotate_rev_reg(&mut issuer, &cred_def, &rev_reg).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &rev_reg_2, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_2.rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_establish_connection_via_public_invite() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution_to_consumer.send_generic_message(get_main_wallet_handle(), "Hello Alice, Faber here").await.unwrap();

        consumer.activate().await.unwrap();
        let consumer_msgs = consumer_to_institution.download_messages(&get_main_agency_client().unwrap(), Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(consumer_msgs.len(), 1);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_oob_connection_bootstrap() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        institution.activate().await.unwrap();
        let request_sender = create_proof_request(&mut institution, REQUESTED_ATTRIBUTES, "[]", "{}", None).await;

        let service = institution.agent.service(&get_main_agency_client().unwrap()).unwrap();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(&GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&ServiceResolvable::FullService(service))
            .append_handshake_protocol(&HandshakeProtocol::ConnectionV1).unwrap()
            .append_a2a_message(request_sender.to_a2a_message()).unwrap();
        let oob_msg = oob_sender.to_a2a_message();

        consumer.activate().await.unwrap();
        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let conns = vec![];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_none());
        let mut conn_receiver = oob_receiver.build_connection(&get_main_agency_client().unwrap(), true).await.unwrap();
        conn_receiver.connect(get_main_wallet_handle(), &get_main_agency_client().unwrap()).await.unwrap();
        conn_receiver.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap()).await.unwrap();
        assert_eq!(ConnectionState::Invitee(InviteeState::Requested), conn_receiver.get_state());
        assert_eq!(oob_sender.oob.id.0, oob_receiver.oob.id.0);

        let conn_sender = connect_using_request_sent_to_public_agent(&mut consumer, &mut institution, &mut conn_receiver).await;

        let (conn_receiver_pw1, _conn_sender_pw1) = create_connected_connections(&mut consumer, &mut institution).await;
        let (conn_receiver_pw2, _conn_sender_pw2) = create_connected_connections(&mut consumer, &mut institution).await;

        let conns = vec![&conn_receiver, &conn_receiver_pw1, &conn_receiver_pw2];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_some());
        assert!(*conn.unwrap() == conn_receiver);

        let conns = vec![&conn_receiver_pw1, &conn_receiver_pw2];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_none());

        let a2a_msg = oob_receiver.extract_a2a_message().unwrap().unwrap();
        assert!(matches!(a2a_msg, A2AMessage::PresentationRequest(..)));
        if let A2AMessage::PresentationRequest(request_receiver) = a2a_msg {
            assert_eq!(request_receiver.request_presentations_attach, request_sender.request_presentations_attach);
        }

        conn_sender.send_generic_message(get_main_wallet_handle(), "Hello oob receiver, from oob sender").await.unwrap();
        consumer.activate().await.unwrap();
        conn_receiver.send_generic_message(get_main_wallet_handle(), "Hello oob sender, from oob receiver").await.unwrap();
        institution.activate().await.unwrap();
        let sender_msgs = conn_sender.download_messages(&get_main_agency_client().unwrap(), None, None).await.unwrap();
        consumer.activate().await.unwrap();
        let receiver_msgs = conn_receiver.download_messages(&get_main_agency_client().unwrap(), None, None).await.unwrap();
        assert_eq!(sender_msgs.len(), 2);
        assert_eq!(receiver_msgs.len(), 2);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_oob_connection_reuse() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution.activate().await.unwrap();
        let service = institution.agent.service(&get_main_agency_client().unwrap()).unwrap();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(&GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&ServiceResolvable::FullService(service));
        let oob_msg = oob_sender.to_a2a_message();

        consumer.activate().await.unwrap();
        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let conns = vec![&consumer_to_institution];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_some());
        conn.unwrap().send_generic_message(get_main_wallet_handle(), "Hello oob sender, from oob receiver").await.unwrap();

        institution.activate().await.unwrap();
        let msgs = institution_to_consumer.download_messages(&get_main_agency_client().unwrap(), None, None).await.unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_oob_connection_handshake_reuse() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (mut consumer_to_institution, mut institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution.activate().await.unwrap();
        let service = institution.agent.service(&get_main_agency_client().unwrap()).unwrap();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(&GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&ServiceResolvable::FullService(service));
        let sender_oob_id = oob_sender.get_id();
        let oob_msg = oob_sender.to_a2a_message();

        consumer.activate().await.unwrap();
        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let conns = vec![&consumer_to_institution];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_some());
        let receiver_oob_id = oob_receiver.get_id();
        let receiver_msg = serde_json::to_string(&oob_receiver.to_a2a_message()).unwrap();
        conn.unwrap().send_handshake_reuse(get_main_wallet_handle(), &receiver_msg).await.unwrap();

        institution.activate().await.unwrap();
        let mut msgs = institution_to_consumer.download_messages(&get_main_agency_client().unwrap(), Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(msgs.len(), 1);
        let reuse_msg = match serde_json::from_str::<A2AMessage>(&msgs.pop().unwrap().decrypted_msg).unwrap() {
            A2AMessage::OutOfBandHandshakeReuse(ref a2a_msg) => {
                assert_eq!(sender_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(receiver_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(a2a_msg.id.0, a2a_msg.thread.thid.as_ref().unwrap().to_string());
                a2a_msg.clone()
            }
            _ => { panic!("Expected OutOfBandHandshakeReuse message type"); }
        };
        institution_to_consumer.update_state_with_message(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &A2AMessage::OutOfBandHandshakeReuse(reuse_msg.clone())).await.unwrap();

        consumer.activate().await.unwrap();
        let mut msgs = consumer_to_institution.download_messages(&get_main_agency_client().unwrap(), Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(msgs.len(), 1);
        let reuse_ack_msg = match serde_json::from_str::<A2AMessage>(&msgs.pop().unwrap().decrypted_msg).unwrap() {
            A2AMessage::OutOfBandHandshakeReuseAccepted(ref a2a_msg) => {
                assert_eq!(sender_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(receiver_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(reuse_msg.id.0, a2a_msg.thread.thid.as_ref().unwrap().to_string());
                a2a_msg.clone()
            }
            _ => { panic!("Expected OutOfBandHandshakeReuseAccepted message type"); }
        };
        consumer_to_institution.update_state_with_message(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &A2AMessage::OutOfBandHandshakeReuseAccepted(reuse_ack_msg)).await.unwrap();
        consumer_to_institution.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap()).await.unwrap();
        assert_eq!(consumer_to_institution.download_messages(&get_main_agency_client().unwrap(), Some(vec![MessageStatusCode::Received]), None).await.unwrap().len(), 0);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_two_enterprise_connections() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;

        let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
        let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_credential_exchange_via_proposal() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let tails_file = rev_reg.get_tails_dir();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_credential_exchange_via_proposal_failed() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let tails_file = rev_reg.get_tails_dir();

        let mut holder = send_cred_proposal(&mut consumer, &consumer_to_institution, &schema_id, &cred_def_id, "comment").await;
        let mut issuer = accept_cred_proposal(&mut institution, &institution_to_consumer, rev_reg_id, Some(tails_file)).await;
        decline_offer(&mut consumer, &consumer_to_institution, &mut holder).await;
        institution.activate().await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        issuer.update_state(get_main_wallet_handle(), &get_main_agency_client().unwrap(), &institution_to_consumer).await.unwrap();
        assert_eq!(IssuerState::Failed, issuer.get_state());
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_credential_exchange_via_proposal_with_negotiation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let tails_file = rev_reg.get_tails_dir();

        let mut holder = send_cred_proposal(&mut consumer, &consumer_to_institution, &schema_id, &cred_def_id, "comment").await;
        let mut issuer = accept_cred_proposal(&mut institution, &institution_to_consumer, rev_reg_id.clone(), Some(tails_file.clone())).await;
        send_cred_proposal_1(&mut holder, &mut consumer, &consumer_to_institution, &schema_id, &cred_def_id, "comment").await;
        accept_cred_proposal_1(&mut issuer, &mut institution, &institution_to_consumer, rev_reg_id, Some(tails_file)).await;
        accept_offer(&mut consumer, &consumer_to_institution, &mut holder).await;
        send_credential(&mut consumer, &mut institution, &mut issuer, &institution_to_consumer, &consumer_to_institution, &mut holder, true).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_presentation_via_proposal() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let tails_file = rev_reg.get_tails_dir();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
        let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
        let mut verifier = Verifier::create("1").unwrap();
        accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
        let selected_credentials_str = prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
        generate_and_send_proof_boo(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
        verify_proof(&mut institution, &mut verifier, &institution_to_consumer).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_presentation_via_proposal_with_rejection() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let tails_file = rev_reg.get_tails_dir();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
        let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
        reject_proof_proposal(&mut institution, &institution_to_consumer).await;
        receive_proof_proposal_rejection(&mut consumer, &mut prover, &consumer_to_institution).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_presentation_via_proposal_with_negotiation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) = _create_address_schema(institution.wallet_handle).await;
        let tails_file = rev_reg.get_tails_dir();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
        let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
        let mut verifier = Verifier::create("1").unwrap();
        accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
        send_proof_proposal_1(&mut consumer, &mut prover, &consumer_to_institution, &cred_def_id).await;
        accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
        let selected_credentials_str = prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
        generate_and_send_proof_boo(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
        verify_proof(&mut institution, &mut verifier, &institution_to_consumer).await;
    }

    pub struct Pool {}

    impl Pool {
        pub fn open() -> Pool {
            futures::executor::block_on(libindy::utils::pool::test_utils::open_test_pool());
            Pool {}
        }
    }

    impl Drop for Pool {
        fn drop(&mut self) {
            futures::executor::block_on(libindy::utils::pool::close()).unwrap();
            futures::executor::block_on(libindy::utils::pool::test_utils::delete_test_pool());
        }
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo() {
        let _setup = SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_nonrevocable_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        // Credential issuance
        faber.offer_non_revocable_credential().await;
        alice.accept_offer().await;
        faber.send_credential().await;
        alice.accept_credential().await;

        // Credential Presentation
        faber.request_presentation().await;
        alice.send_presentation().await;
        faber.verify_presentation().await;
        alice.ensure_presentation_verified().await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_handle_connection_related_messages() {
        let _setup = SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_nonrevocable_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        // Ping
        faber.ping().await;

        alice.update_state(4).await;

        faber.update_state(4).await;

        let faber_connection_info = faber.connection_info().await;
        assert!(faber_connection_info["their"]["protocols"].as_array().is_none());

        // Discovery Features
        faber.discovery_features().await;

        alice.update_state(4).await;

        faber.update_state(4).await;

        let faber_connection_info = faber.connection_info().await;
        assert!(faber_connection_info["their"]["protocols"].as_array().unwrap().len() > 0);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_create_with_message_id_flow() {
        let _setup = SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_nonrevocable_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        /*
         Create with message id flow
        */

        // Credential issuance
        faber.offer_non_revocable_credential().await;

        // Alice creates Credential object with message id
        {
            let message = alice.download_message(PayloadKinds::CredOffer).await.unwrap();
            let cred_offer = alice.get_credential_offer_by_msg_id(&message.uid).await.unwrap();
            alice.credential = Holder::create_from_offer("test", cred_offer).unwrap();

            let pw_did = alice.connection.pairwise_info().pw_did.to_string();
            alice.credential.send_request(get_main_wallet_handle(), pw_did, alice.connection.send_message_closure(get_main_wallet_handle()).unwrap()).await.unwrap();
            assert_eq!(HolderState::RequestSent, alice.credential.get_state());
        }

        faber.send_credential().await;
        alice.accept_credential().await;

        // Credential Presentation
        faber.request_presentation().await;

        // Alice creates Presentation object with message id
        {
            let message = alice.download_message(PayloadKinds::ProofRequest).await.unwrap();
            let presentation_request = alice.get_proof_request_by_msg_id(&message.uid).await.unwrap();
            alice.prover = Prover::create_from_request("test", presentation_request).unwrap();

            let credentials = alice.get_credentials_for_presentation().await;

            alice.prover.generate_presentation(get_main_wallet_handle(), credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

            alice.prover.send_presentation(get_main_wallet_handle(), alice.connection.send_message_closure(get_main_wallet_handle()).unwrap()).await.unwrap();
            assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
        }

        faber.verify_presentation().await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_download_message_flow() {
        SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_nonrevocable_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        /*
         Create with message flow
        */

        // Credential issuance
        faber.offer_non_revocable_credential().await;

        // Alice creates Credential object with Offer
        {
            let message = alice.download_message(PayloadKinds::CredOffer).await.unwrap();

            let cred_offer: CredentialOffer = serde_json::from_str(&message.decrypted_msg).unwrap();
            alice.credential = Holder::create_from_offer("test", cred_offer).unwrap();

            alice.connection.update_message_status(&message.uid, &get_main_agency_client().unwrap()).await.unwrap();

            let pw_did = alice.connection.pairwise_info().pw_did.to_string();
            alice.credential.send_request(get_main_wallet_handle(), pw_did, alice.connection.send_message_closure(get_main_wallet_handle()).unwrap()).await.unwrap();
            assert_eq!(HolderState::RequestSent, alice.credential.get_state());
        }

        faber.send_credential().await;
        alice.accept_credential().await;

        // Credential Presentation
        faber.request_presentation().await;

        // Alice creates Presentation object with Proof Request
        {
            let agency_msg = alice.download_message(PayloadKinds::ProofRequest).await.unwrap();

            let presentation_request: PresentationRequest = serde_json::from_str(&agency_msg.decrypted_msg).unwrap();
            alice.prover = Prover::create_from_request("test", presentation_request).unwrap();

            alice.connection.update_message_status(&agency_msg.uid, &get_main_agency_client().unwrap()).await.unwrap();

            let credentials = alice.get_credentials_for_presentation().await;

            alice.prover.generate_presentation(get_main_wallet_handle(), credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

            alice.prover.send_presentation(get_main_wallet_handle(), alice.connection.send_message_closure(get_main_wallet_handle()).unwrap()).await.unwrap();
            assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
        }

        faber.verify_presentation().await;
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_deserialize_connection_inviter_completed() {
        let _setup = SetupMocks::init();

        let connection = Connection::from_string(CONNECTION_SM_INVITER_COMPLETED).unwrap();
        let _second_string = connection.to_string();

        assert_eq!(connection.pairwise_info().pw_did, "2ZHFFhzA2XtTD6hJqzL7ux");
        assert_eq!(connection.pairwise_info().pw_vk, "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj");
        assert_eq!(connection.cloud_agent_info().agent_did, "EZrZyu4bfydm4ByNm56kPP");
        assert_eq!(connection.cloud_agent_info().agent_vk, "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2");
        assert_eq!(connection.get_state(), ConnectionState::Inviter(InviterState::Completed));
    }

    fn test_deserialize_and_serialize(sm_serialized: &str) {
        let original_object: Value = serde_json::from_str(sm_serialized).unwrap();
        let connection = Connection::from_string(sm_serialized).unwrap();
        let reserialized = connection.to_string().unwrap();
        let reserialized_object: Value = serde_json::from_str(&reserialized).unwrap();

        assert_eq!(original_object, reserialized_object);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_deserialize_and_serialize_should_produce_the_same_object() {
        let _setup = SetupMocks::init();

        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_INVITED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_REQUESTED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_COMPLETED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_COMPLETED);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let connection = Connection::create("test_serialize_deserialize", true, &get_main_agency_client().unwrap()).await.unwrap();
        let first_string = connection.to_string().unwrap();

        let connection2 = Connection::from_string(&first_string).unwrap();
        let second_string = connection2.to_string().unwrap();

        assert_eq!(first_string, second_string);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_serialize_deserialize_serde() {
        let _setup = SetupMocks::init();

        let connection = Connection::create("test_serialize_deserialize", true, &get_main_agency_client().unwrap()).await.unwrap();
        let first_string = serde_json::to_string(&connection).unwrap();

        let connection: Connection = serde_json::from_str(&first_string).unwrap();
        let second_string = serde_json::to_string(&connection).unwrap();
        assert_eq!(first_string, second_string);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_get_credential_def() {
        let setup = SetupWithWalletAndAgency::init().await;
        let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(setup.wallet_handle, utils::constants::DEFAULT_SCHEMA_ATTRS).await;

        let (id, r_cred_def_json) = libindy::utils::anoncreds::get_cred_def_json(setup.wallet_handle, &cred_def_id).await.unwrap();

        assert_eq!(id, cred_def_id);
        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }
}
