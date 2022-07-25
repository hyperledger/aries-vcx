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
#[cfg(feature = "agency_pool_tests")]
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

    use crate::utils::devsetup_agent::test_utils::{Alice, Faber, PayloadKinds, TestAgent};
    use crate::utils::scenarios::test_utils::{_create_address_schema, _exchange_credential, _exchange_credential_with_proposal, accept_cred_proposal, accept_cred_proposal_1, accept_offer, accept_proof_proposal, attr_names, connect_using_request_sent_to_public_agent, create_and_send_nonrevocable_cred_offer, create_connected_connections, create_connected_connections_via_public_invite, create_proof, create_proof_request, decline_offer, generate_and_send_proof, issue_address_credential, prover_select_credentials, prover_select_credentials_and_fail_to_generate_proof, prover_select_credentials_and_send_proof, publish_revocation, receive_proof_proposal_rejection, reject_proof_proposal, requested_attrs, retrieved_to_selected_credentials_simple, revoke_credential, revoke_credential_local, rotate_rev_reg, send_cred_proposal, send_cred_proposal_1, send_cred_req, send_credential, send_proof_proposal, send_proof_proposal_1, send_proof_request, verifier_create_proof_and_send_request, verify_proof};
    use crate::utils::test_macros::ProofStateType;

    use super::*;


    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_establish_connection_via_public_invite() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution_to_consumer.send_generic_message(institution.wallet_handle, "Hello Alice, Faber here").await.unwrap();

        consumer.activate().await.unwrap();
        let consumer_msgs = consumer_to_institution.download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None).await.unwrap();
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

        let service = institution.agent.service(&institution.agency_client).unwrap();
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
        let mut conn_receiver = oob_receiver.build_connection(&consumer.agency_client, true).await.unwrap();
        conn_receiver.connect(consumer.wallet_handle, &consumer.agency_client).await.unwrap();
        conn_receiver.update_state(consumer.wallet_handle, &consumer.agency_client).await.unwrap();
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

        conn_sender.send_generic_message(institution.wallet_handle, "Hello oob receiver, from oob sender").await.unwrap();
        consumer.activate().await.unwrap();
        conn_receiver.send_generic_message(consumer.wallet_handle, "Hello oob sender, from oob receiver").await.unwrap();
        institution.activate().await.unwrap();
        let sender_msgs = conn_sender.download_messages(&institution.agency_client, None, None).await.unwrap();
        consumer.activate().await.unwrap();
        let receiver_msgs = conn_receiver.download_messages(&consumer.agency_client, None, None).await.unwrap();
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
        let service = institution.agent.service(&institution.agency_client).unwrap();
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
        conn.unwrap().send_generic_message(consumer.wallet_handle, "Hello oob sender, from oob receiver").await.unwrap();

        institution.activate().await.unwrap();
        let msgs = institution_to_consumer.download_messages(&institution.agency_client, None, None).await.unwrap();
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
        let service = institution.agent.service(&institution.agency_client).unwrap();
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
        conn.unwrap().send_handshake_reuse(consumer.wallet_handle, &receiver_msg).await.unwrap();

        institution.activate().await.unwrap();
        let mut msgs = institution_to_consumer.download_messages(&institution.agency_client, Some(vec![MessageStatusCode::Received]), None).await.unwrap();
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
        institution_to_consumer.update_state_with_message(institution.wallet_handle, &institution.agency_client, &A2AMessage::OutOfBandHandshakeReuse(reuse_msg.clone())).await.unwrap();

        consumer.activate().await.unwrap();
        let mut msgs = consumer_to_institution.download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None).await.unwrap();
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
        consumer_to_institution.update_state_with_message(consumer.wallet_handle, &consumer.agency_client, &A2AMessage::OutOfBandHandshakeReuseAccepted(reuse_ack_msg)).await.unwrap();
        consumer_to_institution.update_state(consumer.wallet_handle, &consumer.agency_client).await.unwrap();
        assert_eq!(consumer_to_institution.download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None).await.unwrap().len(), 0);
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

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_handle_connection_related_messages() {
        let _setup = SetupEmpty::init();

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
}
