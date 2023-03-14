#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

pub mod utils;

#[cfg(test)]
#[cfg(feature = "agency_pool_tests")]
mod integration_tests {
    use aries_vcx::{
        agency_client::MessageStatusCode,
        common::ledger::transactions::into_did_doc,
        handlers::{
            connection::mediated_connection::ConnectionState,
            out_of_band::{receiver::OutOfBandReceiver, sender::OutOfBandSender},
        },
        messages::{
            a2a::A2AMessage,
            protocols::out_of_band::{GoalCode, HandshakeProtocol},
        },
        protocols::mediated_connection::invitee::state_machine::InviteeState,
        utils::{devsetup::*, mockdata::mockdata_proof::REQUESTED_ATTRIBUTES},
    };
    use async_channel::bounded;
    use messages::protocols::{connection::did::Did, out_of_band::service_oob::ServiceOob};

    use super::*;
    use crate::utils::{
        devsetup_agent::test_utils::{create_test_alice_instance, Faber},
        scenarios::test_utils::{
            _send_message, connect_using_request_sent_to_public_agent, create_connected_connections,
            create_connected_connections_via_public_invite, create_proof_request,
        },
    };

    #[tokio::test]
    async fn test_establish_connection_via_public_invite() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer = create_test_alice_instance(&setup).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

            institution_to_consumer
                .send_generic_message(&institution.profile, "Hello Alice, Faber here")
                .await
                .unwrap();

            let consumer_msgs = consumer_to_institution
                .download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            assert_eq!(consumer_msgs.len(), 1);
        })
        .await;
    }

    #[tokio::test]
    async fn test_oob_connection_bootstrap() {
        use messages::protocols::connection::invite::Invitation;

        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer = create_test_alice_instance(&setup).await;
            let (sender, receiver) = bounded::<A2AMessage>(1);

            let request_sender = create_proof_request(&mut institution, REQUESTED_ATTRIBUTES, "[]", "{}", None).await;

            let did = institution.config_issuer.institution_did.clone();
            let oob_sender = OutOfBandSender::create()
                .set_label("test-label")
                .set_goal_code(&GoalCode::P2PMessaging)
                .set_goal("To exchange message")
                .append_service(&ServiceOob::Did(Did::new(&did).unwrap()))
                .append_handshake_protocol(&HandshakeProtocol::ConnectionV1)
                .unwrap()
                .append_a2a_message(request_sender.to_a2a_message())
                .unwrap();
            let invitation = Invitation::OutOfBand(oob_sender.oob.clone());
            let ddo = into_did_doc(&consumer.profile, &invitation).await.unwrap();
            let oob_msg = oob_sender.to_a2a_message();

            let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
            let conns = vec![];
            let conn = oob_receiver.connection_exists(&consumer.profile, &conns).await.unwrap();
            assert!(conn.is_none());
            let mut conn_receiver = oob_receiver
                .build_connection(&consumer.profile, &consumer.agency_client, ddo, true)
                .await
                .unwrap();
            conn_receiver
                .connect(&consumer.profile, &consumer.agency_client, _send_message(sender))
                .await
                .unwrap();
            assert_eq!(
                ConnectionState::Invitee(InviteeState::Requested),
                conn_receiver.get_state()
            );
            assert_eq!(oob_sender.oob.id.0, oob_receiver.oob.id.0);

            let request = if let A2AMessage::ConnectionRequest(request) = receiver.recv().await.unwrap() {
                request
            } else {
                panic!("Received invalid message type")
            };
            let conn_sender = connect_using_request_sent_to_public_agent(
                &mut consumer,
                &mut institution,
                &mut conn_receiver,
                request,
            )
            .await;

            let (conn_receiver_pw1, _conn_sender_pw1) =
                create_connected_connections(&mut consumer, &mut institution).await;
            let (conn_receiver_pw2, _conn_sender_pw2) =
                create_connected_connections(&mut consumer, &mut institution).await;

            let conns = vec![&conn_receiver, &conn_receiver_pw1, &conn_receiver_pw2];
            let conn = oob_receiver.connection_exists(&consumer.profile, &conns).await.unwrap();
            assert!(conn.is_some());
            assert!(*conn.unwrap() == conn_receiver);

            let conns = vec![&conn_receiver_pw1, &conn_receiver_pw2];
            let conn = oob_receiver.connection_exists(&consumer.profile, &conns).await.unwrap();
            assert!(conn.is_none());

            let a2a_msg = oob_receiver.extract_a2a_message().unwrap().unwrap();
            assert!(matches!(a2a_msg, A2AMessage::PresentationRequest(..)));
            if let A2AMessage::PresentationRequest(request_receiver) = a2a_msg {
                assert_eq!(
                    request_receiver.request_presentations_attach,
                    request_sender.request_presentations_attach
                );
            }

            conn_sender
                .send_generic_message(&institution.profile, "Hello oob receiver, from oob sender")
                .await
                .unwrap();
            conn_receiver
                .send_generic_message(&consumer.profile, "Hello oob sender, from oob receiver")
                .await
                .unwrap();
            let sender_msgs = conn_sender
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            let receiver_msgs = conn_receiver
                .download_messages(&consumer.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(sender_msgs.len(), 2);
            assert_eq!(receiver_msgs.len(), 2);
        })
        .await;
    }

    #[tokio::test]
    async fn test_oob_connection_reuse() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer = create_test_alice_instance(&setup).await;

            let (consumer_to_institution, institution_to_consumer) =
                create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

            let did = institution.config_issuer.institution_did.clone();
            let oob_sender = OutOfBandSender::create()
                .set_label("test-label")
                .set_goal_code(&GoalCode::P2PMessaging)
                .set_goal("To exchange message")
                .append_service(&ServiceOob::Did(Did::new(&did).unwrap()));
            let oob_msg = oob_sender.to_a2a_message();

            let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
            let conns = vec![&consumer_to_institution];
            let conn = oob_receiver.connection_exists(&consumer.profile, &conns).await.unwrap();
            assert!(conn.is_some());
            conn.unwrap()
                .send_generic_message(&consumer.profile, "Hello oob sender, from oob receiver")
                .await
                .unwrap();

            let msgs = institution_to_consumer
                .download_messages(&institution.agency_client, None, None)
                .await
                .unwrap();
            assert_eq!(msgs.len(), 2);
        })
        .await;
    }

    #[tokio::test]
    async fn test_oob_connection_handshake_reuse() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer = create_test_alice_instance(&setup).await;

            let (mut consumer_to_institution, mut institution_to_consumer) =
                create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

            let did = institution.config_issuer.institution_did.clone();
            let oob_sender = OutOfBandSender::create()
                .set_label("test-label")
                .set_goal_code(&GoalCode::P2PMessaging)
                .set_goal("To exchange message")
                .append_service(&ServiceOob::Did(Did::new(&did).unwrap()));
            let sender_oob_id = oob_sender.get_id();
            let oob_msg = oob_sender.to_a2a_message();

            let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
            let conns = vec![&consumer_to_institution];
            let conn = oob_receiver.connection_exists(&consumer.profile, &conns).await.unwrap();
            assert!(conn.is_some());
            let receiver_oob_id = oob_receiver.get_id();
            let receiver_msg = serde_json::to_string(&oob_receiver.to_a2a_message()).unwrap();
            conn.unwrap()
                .send_handshake_reuse(&consumer.profile, &receiver_msg)
                .await
                .unwrap();

            let mut msgs = institution_to_consumer
                .download_messages(
                    &institution.agency_client,
                    Some(vec![MessageStatusCode::Received]),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(msgs.len(), 1);
            let reuse_msg = match serde_json::from_str::<A2AMessage>(&msgs.pop().unwrap().decrypted_msg).unwrap() {
                A2AMessage::OutOfBandHandshakeReuse(ref a2a_msg) => {
                    assert_eq!(sender_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                    assert_eq!(receiver_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                    assert_eq!(a2a_msg.id.0, a2a_msg.thread.thid.as_ref().unwrap().to_string());
                    a2a_msg.clone()
                }
                _ => {
                    panic!("Expected OutOfBandHandshakeReuse message type");
                }
            };
            institution_to_consumer
                .handle_message(
                    A2AMessage::OutOfBandHandshakeReuse(reuse_msg.clone()),
                    &institution.profile,
                )
                .await
                .unwrap();

            let mut msgs = consumer_to_institution
                .download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None)
                .await
                .unwrap();
            assert_eq!(msgs.len(), 1);
            let _reuse_ack_msg = match serde_json::from_str::<A2AMessage>(&msgs.pop().unwrap().decrypted_msg).unwrap() {
                A2AMessage::OutOfBandHandshakeReuseAccepted(ref a2a_msg) => {
                    assert_eq!(sender_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                    assert_eq!(receiver_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                    assert_eq!(reuse_msg.id.0, a2a_msg.thread.thid.as_ref().unwrap().to_string());
                    a2a_msg.clone()
                }
                _ => {
                    panic!("Expected OutOfBandHandshakeReuseAccepted message type");
                }
            };
            consumer_to_institution
                .find_and_handle_message(&consumer.profile, &consumer.agency_client)
                .await
                .unwrap();
            assert_eq!(
                consumer_to_institution
                    .download_messages(&consumer.agency_client, Some(vec![MessageStatusCode::Received]), None)
                    .await
                    .unwrap()
                    .len(),
                0
            );
        })
        .await;
    }

    #[tokio::test]
    pub async fn test_two_enterprise_connections() {
        SetupPool::run(|setup| async move {
            let mut institution = Faber::setup(setup.pool_handle).await;
            let mut consumer1 = create_test_alice_instance(&setup).await;

            let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
            let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
        })
        .await;
    }

    #[tokio::test]
    async fn aries_demo_handle_connection_related_messages() {
        SetupPool::run(|setup| async move {
            let mut faber = Faber::setup(setup.pool_handle).await;
            let mut alice = create_test_alice_instance(&setup).await;

            // Connection
            let invite = faber.create_invite().await;
            alice.accept_invite(&invite).await;

            faber.update_state(3).await;
            alice.update_state(4).await;
            faber.update_state(4).await;

            // Ping
            faber.ping().await;

            alice.handle_messages().await;

            faber.handle_messages().await;

            let faber_connection_info = faber.connection_info().await;
            assert!(faber_connection_info["their"]["protocols"].as_array().is_none());

            // Discovery Features
            faber.discovery_features().await;

            alice.handle_messages().await;

            faber.handle_messages().await;

            let faber_connection_info = faber.connection_info().await;
            warn!("faber_connection_info: {}", faber_connection_info);
            assert!(faber_connection_info["their"]["protocols"].as_array().unwrap().len() > 0);
        })
        .await;
    }
}
