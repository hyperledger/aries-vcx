extern crate async_trait;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;

pub mod utils;

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
    use aries_vcx::global::settings;
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
    use aries_vcx::libindy::credential_def;
    use aries_vcx::libindy::credential_def::{CredentialDef, CredentialDefConfigBuilder};
    use aries_vcx::libindy::proofs::proof_request_internal::{AttrInfo, NonRevokedInterval, PredicateInfo};
    use aries_vcx::libindy::utils::anoncreds::test_utils::{create_and_store_credential_def, create_and_store_nonrevocable_credential_def, create_and_write_test_schema};
    use aries_vcx::libindy::utils::signus;
    use aries_vcx::libindy::utils::signus::create_and_store_my_did;
    use aries_vcx::libindy::utils::wallet::*;
    use aries_vcx::libindy::wallet::open_wallet;
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
    use aries_vcx::utils::{
        constants::{TAILS_DIR, TEST_TAILS_URL},
        get_temp_dir_path,
    };
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::filters;
    use aries_vcx::utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED, CONNECTION_SM_INVITER_COMPLETED};
    use aries_vcx::utils::mockdata::mockdata_proof::REQUESTED_ATTRIBUTES;

    use crate::utils::devsetup_agent::test::{Alice, Faber, PayloadKinds, TestAgent};
    use crate::utils::scenarios::test_utils::{_create_address_schema, _exchange_credential, _exchange_credential_with_proposal, accept_cred_proposal, accept_cred_proposal_1, accept_offer, accept_proof_proposal, attr_names, connect_using_request_sent_to_public_agent, create_and_send_nonrevocable_cred_offer, create_connected_connections, create_connected_connections_via_public_invite, create_proof, create_proof_request, decline_offer, generate_and_send_proof, issue_address_credential, prover_select_credentials, prover_select_credentials_and_fail_to_generate_proof, prover_select_credentials_and_send_proof, publish_revocation, receive_proof_proposal_rejection, reject_proof_proposal, requested_attrs, retrieved_to_selected_credentials_simple, revoke_credential, revoke_credential_local, rotate_rev_reg, send_cred_proposal, send_cred_proposal_1, send_cred_req, send_credential, send_proof_proposal, send_proof_proposal_1, send_proof_request, verifier_create_proof_and_send_request, verify_proof};
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
        verifier.update_state(institution.wallet_handle, &institution.agency_client, &institution_to_consumer).await.unwrap();
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

        info!("test_proof_with_predicates_should_be_validated :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(institution.wallet_handle, &institution.agency_client, &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
        info!("test_proof_with_predicates_should_be_validated :: verifier received presentation!: {}", verifier.get_presentation_attachment().unwrap());
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
        proof_verifier.update_state(verifier.wallet_handle, &verifier.agency_client, &verifier_to_consumer1).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer2, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer2_to_verifier, None, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(verifier.wallet_handle, &verifier.agency_client, &verifier_to_consumer2).await.unwrap();
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
        proof_verifier.update_state(verifier.wallet_handle, &verifier.agency_client, &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name2, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(verifier.wallet_handle, &verifier.agency_client, &verifier_to_consumer).await.unwrap();
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
        verifier.update_state(institution.wallet_handle, &institution.agency_client, &institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;
        institution.activate().await.unwrap();
        verifier.update_state(institution.wallet_handle, &institution.agency_client, &institution_to_consumer).await.unwrap();
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

        let retrieved_credentials = prover.retrieve_credentials(consumer.wallet_handle).await.unwrap();
        let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, false);

        info!("test_real_proof :: generating and sending proof");
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_issuer, &serde_json::to_string(&selected_credentials).unwrap()).await;
        assert_eq!(ProverState::PresentationSent, prover.get_state());
        assert_eq!(presentation_thread_id, prover.get_thread_id().unwrap());
        assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());

        info!("test_real_proof :: AS INSTITUTION VALIDATE PROOF");
        institution.activate().await.unwrap();
        verifier.update_state(institution.wallet_handle, &institution.agency_client, &issuer_to_consumer).await.unwrap();
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
        proof_verifier.update_state(verifier.wallet_handle, &verifier.agency_client, &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(verifier.wallet_handle, &verifier.agency_client, &verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
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
        issuer.update_state(institution.wallet_handle, &institution.agency_client, &institution_to_consumer).await.unwrap();
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
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
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
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
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
            alice.credential.send_request(alice.wallet_handle, pw_did, alice.connection.send_message_closure(alice.wallet_handle).unwrap()).await.unwrap();
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

            alice.prover.generate_presentation(alice.wallet_handle, credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

            alice.prover.send_presentation(alice.wallet_handle, alice.connection.send_message_closure(alice.wallet_handle).unwrap()).await.unwrap();
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

            alice.connection.update_message_status(&message.uid, &alice.agency_client).await.unwrap();

            let pw_did = alice.connection.pairwise_info().pw_did.to_string();
            alice.credential.send_request(alice.wallet_handle, pw_did, alice.connection.send_message_closure(alice.wallet_handle).unwrap()).await.unwrap();
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

            alice.connection.update_message_status(&agency_msg.uid, &alice.agency_client).await.unwrap();

            let credentials = alice.get_credentials_for_presentation().await;

            alice.prover.generate_presentation(alice.wallet_handle, credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

            alice.prover.send_presentation(alice.wallet_handle, alice.connection.send_message_closure(alice.wallet_handle).unwrap()).await.unwrap();
            assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
        }

        faber.verify_presentation().await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_get_credential_def() {
        let setup = SetupWalletPoolAgency::init().await;
        let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(setup.wallet_handle, utils::constants::DEFAULT_SCHEMA_ATTRS).await;

        let (id, r_cred_def_json) = libindy::utils::anoncreds::get_cred_def_json(setup.wallet_handle, &cred_def_id).await.unwrap();

        assert_eq!(id, cred_def_id);
        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }
}
